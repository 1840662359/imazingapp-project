// scripts/diagnose-sav.js —— 诊断 GVAS 属性布局（开发期工具，不随包发布）
//
// 读取预处理模板中的 Active.sav（Stored 条目，直接取字节），
// 按属性序列扫描并打印每个属性的 name / type / size / 原始字节，
// 用于校准 sav.js 的定位逻辑。
//
// 用法: node scripts/diagnose-sav.js <template.imazingapp>

const fs = require('fs');

// ---- 最小 zip 读取：按名称提取 Stored 条目 ----
function extractEntry(zipBuf, targetName) {
  // EOCD: 0x06054b50
  let eocd = -1;
  for (let i = zipBuf.length - 22; i >= 0; i--) {
    if (zipBuf.readUInt32LE(i) === 0x06054b50) { eocd = i; break; }
  }
  if (eocd < 0) throw new Error('EOCD not found');
  const entryCount = zipBuf.readUInt16LE(eocd + 10);
  const cdOffset = zipBuf.readUInt32LE(eocd + 16);
  let p = cdOffset;
  for (let i = 0; i < entryCount; i++) {
    if (zipBuf.readUInt32LE(p) !== 0x02014b50) throw new Error('bad central dir sig');
    const method = zipBuf.readUInt16LE(p + 10);
    const csize = zipBuf.readUInt32LE(p + 20);
    const usize = zipBuf.readUInt32LE(p + 24);
    const nameLen = zipBuf.readUInt16LE(p + 28);
    const extraLen = zipBuf.readUInt16LE(p + 30);
    const commentLen = zipBuf.readUInt16LE(p + 32);
    const localOffset = zipBuf.readUInt32LE(p + 42);
    const name = zipBuf.toString('utf8', p + 46, p + 46 + nameLen);
    if (name === targetName) {
      if (method !== 0) throw new Error(`条目 ${name} 不是 Stored (method=${method})，请先运行 preprocess`);
      const lh = localOffset;
      const lNameLen = zipBuf.readUInt16LE(lh + 26);
      const lExtraLen = zipBuf.readUInt16LE(lh + 28);
      const dataStart = lh + 30 + lNameLen + lExtraLen;
      return zipBuf.subarray(dataStart, dataStart + usize);
    }
    p += 46 + nameLen + extraLen + commentLen;
  }
  throw new Error(`条目不存在: ${targetName}`);
}

// ---- FString 读取（UE 规则：len 为含 NUL 的字节数）----
function readFString(buf, off) {
  const len = buf.readInt32LE(off);
  if (len < 0) {
    // UTF-16：-len 为含 NUL 的 UTF-16 字符数（本项目模板全 ASCII，防御性报错）
    throw new Error(`FString UTF-16 编码暂不支持 @ 0x${off.toString(16)} (len=${len})`);
  }
  if (len === 0) {
    // 空串（属性列表的结束标记）：仅 4 字节 len，无内容
    return { text: '', consumed: 4 };
  }
  const charBytes = len - 1; // len 含末尾 NUL
  const bytes = buf.subarray(off + 4, off + 4 + charBytes);
  return { text: bytes.toString('utf8'), consumed: 4 + len };
}

// ---- 主流程 ----
const templatePath = process.argv[2];
if (!templatePath) { console.error('用法: node scripts/diagnose-sav.js <template.imazingapp>'); process.exit(1); }

const zipBuf = fs.readFileSync(templatePath);
const sav = extractEntry(zipBuf, 'Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav');
console.log(`Active.sav: ${sav.length} 字节\n`);

// GVAS header（按 uesave 0.7.1 的 Header 结构）
const magic = sav.toString('ascii', 0, 4);
console.log(`magic: ${magic}`);
const saveGameVersion = sav.readUInt32LE(4);
const packageVersion = sav.readUInt32LE(8);
const engineMajor = sav.readUInt16LE(12);
const engineMinor = sav.readUInt16LE(14);
const enginePatch = sav.readUInt16LE(16);
const engineBuild = sav.readUInt32LE(18);
console.log(`saveGameVersion=${saveGameVersion} packageVersion=${packageVersion}`);
console.log(`engine: ${engineMajor}.${engineMinor}.${enginePatch} build=${engineBuild}`);

let p = 22;
const { text: engineVersionStr, consumed: c0 } = readFString(sav, p);
p += c0;
console.log(`engineVersion FString: "${engineVersionStr}"`);

const customVersionFormat = sav.readUInt32LE(p);
p += 4;
const customVersionCount = sav.readUInt32LE(p);
p += 4;
console.log(`customVersionFormat=${customVersionFormat} count=${customVersionCount}`);

// custom version 条目：每条 16B GUID + 4B int32
for (let i = 0; i < customVersionCount; i++) {
  p += 20;
}

// UE 4.18 GVAS：customVersion 之后先读根 struct 名（FString），再读属性列表
const { text: rootStruct, consumed: cRoot } = readFString(sav, p);
p += cRoot;
console.log(`根 struct 名: "${rootStruct}"`);
console.log(`\n属性序列起始偏移: 0x${p.toString(16)} (${p})\n`);

// ---- tag 特有 data 的大小（老格式 GVAS）----
function tagDataSize(type) {
  switch (type) {
    case 'BoolProperty': return 1;              // u8
    case 'ByteProperty':                        // FString（枚举类型名）
    case 'EnumProperty': return null;           // FString（枚举类型名）→ 动态
    case 'StructProperty': return null;         // FString + FGuid(16B) → 动态
    case 'ArrayProperty':                       // 内层 type FString
    case 'SetProperty': return null;            // key type FString → 动态
    case 'MapProperty': return null;            // key + value type FString → 动态
    default: return 0;                          // Int/Float/Str 等：无 data
  }
}

// 属性扫描
let idx = 0;
let guard = 0;
const targets = ['FPSLevel', 'FpsLevel_TA', 'ScreenLightness', 'TpViewValue', 'JoystickSprintParam'];
const found = {};
while (guard++ < 500) {
  const tagOff = p;
  const { text: name, consumed: c1 } = readFString(sav, p);
  p += c1;
  if (name === 'None' || c1 === 4) {
    // "None" 属性 → 属性列表结束
    console.log(`\n[结束] @ 0x${tagOff.toString(16)}`);
    break;
  }
  const { text: type, consumed: c2 } = readFString(sav, p);
  p += c2;
  const size = sav.readUInt32LE(p);
  p += 4;
  const index = sav.readUInt32LE(p);
  p += 4;

  // tag 特有 data
  let dataBytes = 0;
  let dataHex = '';
  if (type === 'BoolProperty') {
    dataBytes = 2; // 1 字节值 + 1 字节填充（实测对齐到 2）
    dataHex = `${sav[p].toString(16).padStart(2, '0')} ${sav[p + 1].toString(16).padStart(2, '0')}`;
    p += 2;
  } else if (type === 'ByteProperty' || type === 'EnumProperty' || type === 'ArrayProperty' || type === 'SetProperty') {
    const { consumed: c3 } = readFString(sav, p);
    p += c3;
    dataBytes = c3;
  } else if (type === 'MapProperty') {
    const { consumed: c3 } = readFString(sav, p);
    p += c3;
    const { consumed: c4 } = readFString(sav, p);
    p += c4;
    dataBytes = c3 + c4;
  } else if (type === 'StructProperty') {
    const { consumed: c3 } = readFString(sav, p);
    p += c3;
    p += 16; // FGuid
    dataBytes = c3 + 16;
  }

  const valueStart = p;
  const valueBytes = sav.subarray(p, p + size);
  p += size;

  if (targets.includes(name)) {
    found[name] = { type, size, index, valueStart };
  }
  if (idx < 8 || targets.includes(name)) {
    console.log(`[${idx}] ${name} (${type}) size=${size} index=${index} tag@0x${tagOff.toString(16)}`);
    console.log(`    data[${dataBytes}${dataHex ? ' ' + dataHex : ''}] 值区 @ 0x${valueStart.toString(16)}: ${Array.from(valueBytes.subarray(0, Math.min(size, 12))).map(b => b.toString(16).padStart(2, '0')).join(' ')}${size > 12 ? '...' : ''}`);
  }
  idx++;
}

console.log(`\n共 ${idx} 个属性，扫描结束 @ 0x${p.toString(16)} (${p}) / 总长 ${sav.length}`);
console.log('\n目标属性定位：');
for (const [name, f] of Object.entries(found)) {
  console.log(`  ${name}: ${f.type} @ 值区 0x${f.valueStart.toString(16)} (${f.valueStart}) size=${f.size} index=${f.index}`);
}
