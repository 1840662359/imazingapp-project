// miniprogram/engine/zip.js —— zip 字节级重写（零压缩/解压代码）。
//
// 前提：模板经 preprocess 预处理后，目标条目（Active.sav / UserCustom.ini）
// 均为 Stored（method=0），其余条目原样字节复制。
// 原理：EOCD → 中央目录 → 逐条目拷贝（修正偏移），修改条目以 Stored 写回
// （重算 CRC32），重建中央目录 + EOCD。
//
// 对外：
//   readZipEntries(u8)          —— 条目清单（Node 测试对比 golden 用）
//   rewriteZip(template, mods)  —— 重写（mods: { entryName: Uint8Array }）

'use strict';

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

/** CRC32（标准 zip 算法）。 */
function crc32(u8) {
  let c = 0xffffffff;
  for (let i = 0; i < u8.length; i++) {
    c = CRC_TABLE[(c ^ u8[i]) & 0xff] ^ (c >>> 8);
  }
  return (c ^ 0xffffffff) >>> 0;
}

/** 小端写入 u32。 */
function writeU32(u8, off, v) {
  u8[off] = v & 0xff;
  u8[off + 1] = (v >>> 8) & 0xff;
  u8[off + 2] = (v >>> 16) & 0xff;
  u8[off + 3] = (v >>> 24) & 0xff;
}

/** 小端写入 u16。 */
function writeU16(u8, off, v) {
  u8[off] = v & 0xff;
  u8[off + 1] = (v >>> 8) & 0xff;
}

/**
 * 解析 zip：返回 { entries, cdStart, cdSize, totalCount, eocd }。
 * entries 每项：{ name, method, flags, time, date, crc32, compressedSize, size,
 *               nameLen, extraLen, commentLen, localOffset, internalAttr, externalAttr }
 */
function readZipEntries(u8) {
  const dv = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);

  // ---- EOCD：从尾部向前找签名 ----
  let eocd = -1;
  const min = u8.length - 22 - 65535; // 注释最长 65535
  for (let i = u8.length - 22; i >= Math.max(0, min); i--) {
    if (dv.getUint32(i, true) === 0x06054b50) { eocd = i; break; }
  }
  if (eocd < 0) throw new Error('zip: EOCD 未找到');

  const totalCount = dv.getUint16(eocd + 10, true);
  const cdSize = dv.getUint32(eocd + 12, true);
  const cdStart = dv.getUint32(eocd + 16, true);
  if (dv.getUint32(cdStart, true) !== 0x02014b50) throw new Error('zip: 中央目录签名错误');

  // ---- 中央目录 ----
  const entries = [];
  let p = cdStart;
  for (let i = 0; i < totalCount; i++) {
    if (dv.getUint32(p, true) !== 0x02014b50) throw new Error(`zip: 中央目录条目 ${i} 签名错误`);
    const versionMadeBy = dv.getUint16(p + 4, true);
    const versionNeeded = dv.getUint16(p + 6, true);
    const flags = dv.getUint16(p + 8, true);
    const method = dv.getUint16(p + 10, true);
    const time = dv.getUint16(p + 12, true);
    const date = dv.getUint16(p + 14, true);
    const crc = dv.getUint32(p + 16, true);
    const compressedSize = dv.getUint32(p + 20, true);
    const size = dv.getUint32(p + 24, true);
    const nameLen = dv.getUint16(p + 28, true);
    const extraLen = dv.getUint16(p + 30, true);
    const commentLen = dv.getUint16(p + 32, true);
    const diskStart = dv.getUint16(p + 34, true);
    const internalAttr = dv.getUint16(p + 36, true);
    const externalAttr = dv.getUint32(p + 38, true);
    const localOffset = dv.getUint32(p + 42, true);
    let name = '';
    for (let j = 0; j < nameLen; j++) name += String.fromCharCode(u8[p + 46 + j]);
    // local header 的 version needed（与中央目录可能不同，各自保留原值）
    if (localOffset + 30 + nameLen + extraLen > u8.length) {
      throw new Error(`zip: 条目 ${name} 的 local header 越界`);
    }
    const localVersionNeeded = dv.getUint16(localOffset + 4, true);
    // local 与 central 的 nameLen/extraLen 必须一致，否则无法按 central 切片复制
    const localNameLen = dv.getUint16(localOffset + 26, true);
    const localExtraLen = dv.getUint16(localOffset + 28, true);
    if (localNameLen !== nameLen || localExtraLen !== extraLen) {
      throw new Error(`zip: 条目 ${name} 的 local/central 头部字段不一致（local ${localNameLen}/${localExtraLen} vs central ${nameLen}/${extraLen}）`);
    }
    entries.push({
      name, flags, method, time, date, crc32: crc,
      compressedSize, size, nameLen, extraLen, commentLen,
      diskStart, internalAttr, externalAttr, localOffset,
      versionMadeBy, versionNeeded, localVersionNeeded,
      cdEntryOffset: p, // 中央目录内条目起点（name/extra/comment 以中央目录为准）
    });
    p += 46 + nameLen + extraLen + commentLen;
  }
  return { entries, cdStart, cdSize, totalCount, eocd };
}

/** 条目是否为目录（zip 规范：目录名以 '/' 结尾）。 */
function isDir(entry) {
  return entry.name.endsWith('/');
}

/**
 * 重写 zip：未修改条目原样字节复制，修改条目以 Stored 写回。
 * @param {Uint8Array} template 模板字节（目标条目须为 Stored）
 * @param {Map<string, Uint8Array>} mods entryName → 新字节
 * @returns {Uint8Array} 新 zip 字节
 */
function rewriteZip(template, mods) {
  const { entries, totalCount, cdSize: _cdSize } = readZipEntries(template);
  const modNames = new Set(mods.keys());
  for (const name of modNames) {
    if (!entries.some((e) => e.name === name)) {
      throw new Error(`zip: 待修改条目不存在于模板: ${name}`);
    }
  }

  // ---- 第一遍：计算输出大小 ----
  const outEntries = []; // { entry, dataStart(in), newBytes?, stored }
  let bodySize = 0;
  for (const entry of entries) {
    const localHeaderLen = 30 + entry.nameLen + entry.extraLen;
    const isMod = mods.has(entry.name);
    let dataLen;
    if (isDir(entry)) {
      dataLen = 0;
    } else if (isMod) {
      dataLen = mods.get(entry.name).length;
    } else {
      dataLen = entry.compressedSize;
    }
    outEntries.push({
      entry,
      localHeaderLen,
      dataLen,
      newBytes: isMod ? mods.get(entry.name) : null,
      localOffset: bodySize, // 新 local header 起点
    });
    bodySize += localHeaderLen + dataLen;
  }

  let cdSize = 0;
  for (const o of outEntries) {
    cdSize += 46 + o.entry.nameLen + o.entry.extraLen + o.entry.commentLen;
  }
  const totalSize = bodySize + cdSize + 22;
  const out = new Uint8Array(totalSize);
  const dv = new DataView(out.buffer, out.byteOffset, out.byteLength);

  // ---- 第二遍：写 local headers + 数据 ----
  let p = 0;
  for (const o of outEntries) {
    const e = o.entry;
    writeU32(out, p, 0x04034b50); // local header sig
    writeU16(out, p + 4, e.localVersionNeeded); // 保留原值
    // 清 data descriptor 标志（bit 3）：新写条目均无 descriptor，避免头尾不一致
    writeU16(out, p + 6, e.flags & ~0x08);
    if (o.newBytes) {
      // 修改条目：Stored 写回
      writeU16(out, p + 8, 0); // method = Stored
      writeU16(out, p + 10, e.time);
      writeU16(out, p + 12, e.date);
      const crc = crc32(o.newBytes);
      writeU32(out, p + 14, crc);
      writeU32(out, p + 18, o.dataLen); // compressedSize
      writeU32(out, p + 22, o.dataLen); // size
    } else {
      // 未修改/目录：原样复制头部字段
      writeU16(out, p + 8, e.method);
      writeU16(out, p + 10, e.time);
      writeU16(out, p + 12, e.date);
      writeU32(out, p + 14, e.crc32);
      writeU32(out, p + 18, e.compressedSize);
      writeU32(out, p + 22, e.size);
    }
    writeU16(out, p + 26, e.nameLen);
    writeU16(out, p + 28, e.extraLen);
    // name + extra：以 local header 的原始段为准（central 目录另算）
    const nameStart = p + 30;
    const srcNameStart = e.localOffset + 30;
    out.set(template.subarray(srcNameStart, srcNameStart + e.nameLen + e.extraLen), nameStart);
    p += 30 + e.nameLen + e.extraLen;

    // 数据
    if (o.newBytes) {
      out.set(o.newBytes, p);
    } else if (!isDir(e)) {
      const srcData = e.localOffset + 30 + e.nameLen + e.extraLen;
      out.set(template.subarray(srcData, srcData + e.compressedSize), p);
    }
    p += o.dataLen;
  }

  // ---- 中央目录 ----
  const cdStart = p;
  for (const o of outEntries) {
    const e = o.entry;
    writeU32(out, p, 0x02014b50);
    writeU16(out, p + 4, e.versionMadeBy); // 保留原值（含系统标识，如 unix）
    writeU16(out, p + 6, e.versionNeeded); // 保留原值
    writeU16(out, p + 8, e.flags & ~0x08); // 与 local header 一致：清 data descriptor 标志
    writeU16(out, p + 10, o.newBytes ? 0 : e.method);
    writeU16(out, p + 12, e.time);
    writeU16(out, p + 14, e.date);
    writeU32(out, p + 16, o.newBytes ? crc32(o.newBytes) : e.crc32);
    writeU32(out, p + 20, o.newBytes ? o.dataLen : e.compressedSize); // compressedSize
    writeU32(out, p + 24, o.newBytes ? o.dataLen : e.size); // size
    writeU16(out, p + 28, e.nameLen);
    writeU16(out, p + 30, e.extraLen);
    writeU16(out, p + 32, e.commentLen);
    writeU16(out, p + 34, 0); // disk start
    writeU16(out, p + 36, e.internalAttr);
    writeU32(out, p + 38, e.externalAttr);
    writeU32(out, p + 42, o.localOffset);
    const nameStart = p + 46;
    const srcNameStart = e.cdEntryOffset + 46;
    out.set(template.subarray(srcNameStart, srcNameStart + e.nameLen + e.extraLen + e.commentLen), nameStart);
    p += 46 + e.nameLen + e.extraLen + e.commentLen;
  }
  const cdEnd = p;

  // ---- EOCD ----
  writeU32(out, p, 0x06054b50);
  writeU16(out, p + 4, 0); // disk number
  writeU16(out, p + 6, 0); // cd start disk
  writeU16(out, p + 8, totalCount);
  writeU16(out, p + 10, totalCount);
  writeU32(out, p + 12, cdEnd - cdStart);
  writeU32(out, p + 16, cdStart);
  writeU16(out, p + 20, 0); // comment len

  return out;
}

module.exports = { readZipEntries, rewriteZip, crc32 };
