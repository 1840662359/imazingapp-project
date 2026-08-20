// miniprogram/engine/sav.js —— UE4 GVAS 存档「定位 + 原地覆盖」。
//
// 不移植完整 GVAS 解析器：模板受控（扫描工具保证属性存在且定长），
// 只需沿属性序列定位目标属性，覆写定长值区（Int/Float = 4 字节）。
//
// 布局规则经 devtools `dump --scan` 实测校准（88 个属性全部验证）：
//   header: magic(4) + saveGameVersion(4) + packageUe4(4) + major(2) + minor(2)
//           + patch(2) + build(4) + engineVersion FString
//           + customFormat(4) + count(4) + count×20 + 根 struct 名 FString
//   属性:   name FString + type FString + size u32 + index u32
//           + data（Bool=2B；Byte/Enum/Array/Set=FString+1B；Struct=FString+17B；
//             Map=2×FString+1B；其它=1B）
//           + 值区 size 字节
//   FString: len i32（含 NUL 的字节数），0 = 空串，负 = UTF-16（不支持）

'use strict';

/** FString 读取：返回 { text, consumed }。 */
function readFString(u8, off) {
  const dv = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
  if (off + 4 > u8.length) throw new Error(`FString 越界 @ 0x${off.toString(16)}`);
  const len = dv.getInt32(off, true);
  if (len < 0) {
    throw new Error(`FString UTF-16 编码不支持 @ 0x${off.toString(16)} (len=${len})`);
  }
  if (len === 0) return { text: '', consumed: 4 };
  const charBytes = len - 1; // len 含末尾 NUL
  if (off + 4 + len > u8.length) {
    throw new Error(`FString 内容越界 @ 0x${off.toString(16)} len=${len}`);
  }
  let text = '';
  for (let i = 0; i < charBytes; i++) {
    text += String.fromCharCode(u8[off + 4 + i]);
  }
  return { text, consumed: 4 + len };
}

/** tag 特有 data 的字节数（含尾部 1 字节对齐/保留；Bool 为 2）。 */
function tagDataSize(u8, off, type) {
  switch (type) {
    case 'BoolProperty':
      return 2; // 1 字节值 + 1 字节对齐（实测）
    case 'ByteProperty':
    case 'EnumProperty':
    case 'ArrayProperty':
    case 'SetProperty':
      return readFString(u8, off).consumed + 1;
    case 'StructProperty':
      return readFString(u8, off).consumed + 16 + 1; // 类型 FString + FGuid + 对齐
    case 'MapProperty': {
      const c1 = readFString(u8, off).consumed;
      const c2 = readFString(u8, off + c1).consumed;
      return c1 + c2 + 1;
    }
    default:
      return 1; // Int/Float/Str 等（实测）
  }
}

/**
 * 解析 GVAS 头，返回属性序列起始偏移。
 */
function headerEnd(u8) {
  const dv = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
  if (u8.length < 24) throw new Error('GVAS 文件过短');
  const magic = String.fromCharCode(u8[0], u8[1], u8[2], u8[3]);
  if (magic !== 'GVAS') throw new Error(`不是 GVAS 存档: ${magic}`);
  const saveGameVersion = dv.getUint32(4, true);
  if (saveGameVersion !== 2) {
    throw new Error(`不支持的 GVAS 版本: ${saveGameVersion}（仅支持 2）`);
  }

  let p = 4 + 4 + 4 + 2 + 2 + 2 + 4; // magic + saveGameVersion + packageUe4 + major/minor/patch + build
  p += readFString(u8, p).consumed; // engineVersion FString
  if (p + 8 > u8.length) {
    throw new Error(`GVAS 头部越界 @ 0x${p.toString(16)}（模板可能损坏）`);
  }
  const customCount = dv.getUint32(p + 4, true);
  p += 4 + 4 + customCount * 20; // customFormat + count + count×20
  p += readFString(u8, p).consumed; // 根 struct 名 FString
  return p;
}

/**
 * 沿属性序列定位目标属性。
 * @param {Uint8Array} u8 GVAS 存档字节
 * @param {string} name 属性名
 * @returns {{ valueOffset: number, size: number, type: string } | null} 未找到返回 null
 */
function findProperty(u8, name) {
  const dv = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
  let p = headerEnd(u8);

  for (let guard = 0; guard < 500; guard++) {
    const { text, consumed } = readFString(u8, p);
    p += consumed;
    if (text === 'None' || text === '') return null; // 属性列表结束

    // 跳过 type / size / index / data
    const typeInfo = readFString(u8, p);
    const type = typeInfo.text;
    p += typeInfo.consumed;
    if (p + 8 > u8.length) {
      throw new Error(`GVAS 属性 ${text} 头越界 @ 0x${p.toString(16)}（模板可能损坏）`);
    }
    const size = dv.getUint32(p, true);
    p += 4; // size
    p += 4; // index
    p += tagDataSize(u8, p, type);

    if (text === name) {
      return { valueOffset: p, size, type };
    }
    p += size; // 跳过值区
  }
  throw new Error('属性扫描未收敛');
}

/**
 * 覆写目标属性的值区（定长覆盖，不改变文件结构）。
 * @param {Uint8Array} savBytes 原始 sav 字节
 * @param {string} name 属性名
 * @param {Uint8Array} valueBytes 新值字节（长度须与值区一致）
 * @returns {Uint8Array} 新 sav 字节
 */
function setValue(savBytes, name, valueBytes) {
  const found = findProperty(savBytes, name);
  if (!found) throw new Error(`sav 属性不存在: ${name}（模板版本可能不兼容）`);
  if (found.size !== valueBytes.length) {
    throw new Error(
      `sav 属性 ${name} 值区大小不匹配: 期望 ${found.size}，实际 ${valueBytes.length}（模板版本可能不兼容）`
    );
  }
  const out = new Uint8Array(savBytes.length);
  out.set(savBytes);
  out.set(valueBytes, found.valueOffset);
  return out;
}

/** 按 TypedValue 构造定长字节（Int/Float = 4 字节 LE）。 */
function valueToBytes(tv) {
  const buf = new ArrayBuffer(4);
  const dv = new DataView(buf);
  switch (tv.t) {
    case 'int':
      dv.setInt32(0, tv.v, true);
      break;
    case 'float':
      dv.setFloat32(0, tv.v, true);
      break;
    default:
      throw new Error(`sav 仅支持 Int/Float 覆盖，got ${tv.t}`);
  }
  return new Uint8Array(buf);
}

module.exports = { setValue, findProperty, valueToBytes };
