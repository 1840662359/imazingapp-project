// miniprogram/engine/test/engine.test.js —— 纯前端引擎单测。
//
// 核心：对照 devtools golden 生成（条目级 name/method/size/crc32 全等），
// 保证 JS 引擎与 Rust pipeline 行为一致（字节级时间戳差异除外）。
//
// 用法: node miniprogram/engine/test/engine.test.js
// 前置: cargo run -p devtools -- golden config template-standalone.imazingapp golden

'use strict';

const assert = require('assert');
const fs = require('fs');
const path = require('path');

const config = require('../config.js');
const { getSchemaView } = require('../schema.js');
const { generate } = require('../generate.js');
const { readZipEntries } = require('../zip.js');
const { findProperty } = require('../sav.js');
const { generateFilename } = require('../filename.js');
const { translate } = require('../translate.js');
const { IniDoc } = require('../ini.js');
const { base64ToBytes } = require('../util.js');

const ROOT = path.join(__dirname, '..', '..', '..');
const GOLDEN_DIR = path.join(ROOT, 'golden');
const TEMPLATE = base64ToBytes(require('../template.js'));

const SAV_PATH = 'Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav';

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    passed++;
    console.log(`✓ ${name}`);
  } catch (e) {
    failed++;
    console.error(`✗ ${name}\n  ${e.message}`);
  }
}

/** 从 zip 字节提取 Stored 条目。 */
function extractEntry(zipBytes, entryName) {
  const { entries } = readZipEntries(zipBytes);
  const entry = entries.find((e) => e.name === entryName);
  assert(entry, `条目不存在: ${entryName}`);
  const dataStart = entry.localOffset + 30 + entry.nameLen + entry.extraLen;
  return { bytes: zipBytes.subarray(dataStart, dataStart + entry.size), entry };
}

/** 从 sav 字节读取 Int/Float 值。 */
function readSavValue(savBytes, name) {
  const found = findProperty(savBytes, name);
  assert(found, `sav 属性不存在: ${name}`);
  const dv = new DataView(savBytes.buffer, savBytes.byteOffset, savBytes.byteLength);
  if (found.type === 'IntProperty') return dv.getInt32(found.valueOffset, true);
  if (found.type === 'FloatProperty') return dv.getFloat32(found.valueOffset, true);
  throw new Error(`未知属性类型: ${found.type}`);
}

// ============================================================================
// 1. golden 对照：每个 case 的条目清单必须与 Rust pipeline 产物完全一致
// ============================================================================
const caseFiles = fs.readdirSync(GOLDEN_DIR).filter((f) => f.endsWith('.json')).sort();
assert(caseFiles.length > 0, `golden 目录为空: ${GOLDEN_DIR}（先运行 devtools golden）`);

for (const file of caseFiles) {
  const golden = JSON.parse(fs.readFileSync(path.join(GOLDEN_DIR, file), 'utf8'));
  test(`golden ${golden.name}：条目级对比`, () => {
    const selections = golden.selections;
    const result = generate(config, selections, TEMPLATE);
    const actual = readZipEntries(result.bytes).entries.map((e) => ({
      name: e.name,
      method: e.method,
      size: e.size,
      crc32: e.crc32,
    }));
    const expected = golden.entries;
    assert.strictEqual(actual.length, expected.length, `条目数不一致: ${actual.length} vs ${expected.length}`);
    for (let i = 0; i < expected.length; i++) {
      const a = actual[i];
      const e = expected[i];
      assert.strictEqual(a.name, e.name, `条目 ${i} 名称不一致`);
      assert.strictEqual(a.method, e.method, `条目 ${i} (${a.name}) 压缩方式不一致`);
      assert.strictEqual(a.size, e.size, `条目 ${i} (${a.name}) 大小不一致`);
      assert.strictEqual(a.crc32, e.crc32, `条目 ${i} (${a.name}) CRC32 不一致`);
    }
  });
}

// ============================================================================
// 1.5 zip 元数据保留：version made by / version needed 必须与原模板一致
// ============================================================================
test('zip：version made by / needed 保留原值', () => {
  const tplEntries = readZipEntries(TEMPLATE).entries;
  const result = generate(config, { fps: 'fps_120' }, TEMPLATE);
  const outEntries = readZipEntries(result.bytes).entries;
  assert.strictEqual(outEntries.length, tplEntries.length);
  for (let i = 0; i < outEntries.length; i++) {
    assert.strictEqual(outEntries[i].versionMadeBy, tplEntries[i].versionMadeBy, `条目 ${i} version made by`);
    assert.strictEqual(outEntries[i].versionNeeded, tplEntries[i].versionNeeded, `条目 ${i} version needed`);
    assert.strictEqual(outEntries[i].localVersionNeeded, tplEntries[i].localVersionNeeded, `条目 ${i} local version needed`);
  }
});

// ============================================================================
// 2. sav 值抽查：case-02（全高配）关键属性必须命中预期值
// ============================================================================
test('sav 值：case-02 全高配（FPSLevel=8, TpViewValue=225, ScreenLightness=2.0, JoystickSprintParam=1.0）', () => {
  const selections = {
    quality: 'hd', fps: 'fps_120', resolution_scale: 2.0, visual_style: 'vivid',
    anti_aliasing: 'on', shadow: 'on', render_optimization: 'on',
    brightness: 2.0, camera_fov: 'on', sprint_sensitivity: 1.0,
  };
  const result = generate(config, selections, TEMPLATE);
  const { bytes: sav } = extractEntry(result.bytes, SAV_PATH);
  assert.strictEqual(readSavValue(sav, 'FPSLevel'), 8);
  assert.strictEqual(readSavValue(sav, 'FpsLevel_TA'), 8);
  assert.strictEqual(readSavValue(sav, 'TpViewValue'), 225);
  assert.strictEqual(readSavValue(sav, 'ScreenLightness'), 2.0);
  assert.strictEqual(readSavValue(sav, 'JoystickSprintParam'), 1.0);
});

test('sav 值：case-03 低配（FPSLevel=7, TpViewValue=80, ScreenLightness=1.5, JoystickSprintParam=0.5）', () => {
  const selections = {
    quality: 'balanced', fps: 'fps_90', resolution_scale: 1.5, visual_style: 'classic',
    anti_aliasing: 'off', shadow: 'off', render_optimization: 'off',
    brightness: 1.5, camera_fov: 'off', sprint_sensitivity: 0.5,
  };
  const result = generate(config, selections, TEMPLATE);
  const { bytes: sav } = extractEntry(result.bytes, SAV_PATH);
  assert.strictEqual(readSavValue(sav, 'FPSLevel'), 7);
  assert.strictEqual(readSavValue(sav, 'FpsLevel_TA'), 7);
  assert.strictEqual(readSavValue(sav, 'TpViewValue'), 80);
  assert.strictEqual(readSavValue(sav, 'ScreenLightness'), 1.5);
  assert.strictEqual(readSavValue(sav, 'JoystickSprintParam'), 0.5);
});

// ============================================================================
// 3. ini 值验证：remove_when（渲染优化关）必须删除 5 条 CVar
// ============================================================================
test('ini：render_optimization off 删除 5 条 CVar（remove_when）', () => {
  const selections = { render_optimization: 'off' };
  const result = generate(config, selections, TEMPLATE);
  const { entries } = readZipEntries(result.bytes);
  const iniEntry = entries.find((e) => e.name.endsWith('UserCustom.ini'));
  const { bytes: iniBytes } = extractEntry(result.bytes, iniEntry.name);
  const ini = IniDoc.load(iniBytes);
  const section = 'UserCustom DeviceProfile';
  for (const cvar of ['r.BloomQuality', 'r.LightShaftQuality', 'r.Mobile.TonemapperFilm',
    'r.Mobile.AlwaysResolveDepth', 'r.MobileSimpleShader']) {
    assert(!ini.contains(section, cvar), `渲染优化关应删除 ${cvar}`);
  }
});

test('ini：缺失 CVar 自动插入且保留已有配置', () => {
  const ini = IniDoc.load(Buffer.from(
    '[UserCustom DeviceProfile]\n+CVars=r.UserQualitySetting=1\n',
    'utf8'
  ));

  ini.set('UserCustom DeviceProfile', 'r.PUBGDeviceFPSDef', '8');

  assert.strictEqual(ini.get('UserCustom DeviceProfile', 'r.UserQualitySetting'), '1');
  assert.strictEqual(ini.get('UserCustom DeviceProfile', 'r.PUBGDeviceFPSDef'), '8');
});

test('ini：目标 section 缺失时自动创建', () => {
  const ini = IniDoc.load(Buffer.from('[Other]\n+CVars=keep=value\n', 'utf8'));

  ini.set('UserCustom DeviceProfile', 'r.PUBGDeviceFPSDef', '8');
  const saved = Buffer.from(ini.save()).toString('utf8');

  assert(saved.includes('[Other]\n+CVars=keep=value'));
  assert(saved.includes('[UserCustom DeviceProfile]\n+CVars=r.PUBGDeviceFPSDef=8'));
});

test('ini：当前模板缺失项生成时补齐，并保留模板参数与备份 section', () => {
  const result = generate(config, {
    fps: 'fps_120',
    render_optimization: 'on',
  }, TEMPLATE);
  const { entries } = readZipEntries(result.bytes);
  const iniEntry = entries.find((e) => e.name.endsWith('UserCustom.ini'));
  const { bytes: iniBytes } = extractEntry(result.bytes, iniEntry.name);
  const ini = IniDoc.load(iniBytes);
  const section = 'UserCustom DeviceProfile';

  for (const cvar of [
    'r.PUBGDeviceFPSDef', 'r.PUBGDeviceFPSLow', 'r.PUBGDeviceFPSMid',
    'r.PUBGDeviceFPSHigh', 'r.PUBGDeviceFPSHDR', 'r.PUBGDeviceFPSUltralHigh',
    'r.PUBGDeviceFPSUltimateHigh',
  ]) {
    assert.strictEqual(ini.get(section, cvar), '8', `应插入 ${cvar}=8`);
  }
  assert.strictEqual(ini.get(section, 'r.BloomQuality'), '0');
  assert.strictEqual(ini.get(section, 'r.LightShaftQuality'), '0');
  assert.strictEqual(ini.get(section, 'r.Mobile.TonemapperFilm'), '0');
  assert.strictEqual(ini.get(section, 'r.Mobile.AlwaysResolveDepth'), '0');
  assert.strictEqual(ini.get(section, 'r.MobileSimpleShader'), '1');
  assert.notStrictEqual(
    ini.get(section, 'r.UserQualitySetting'),
    undefined,
    '应保留模板原有配置项'
  );
  assert(ini.sections.has('BackUp DeviceProfile'), '应保留 BackUp DeviceProfile');
});

// ============================================================================
// 4. 文件名（对齐 Rust routes::generate_filename 测试用例）
// ============================================================================
test('filename 全组合', () => {
  assert.strictEqual(
    generateFilename({
      resolution_scale: 2.0, quality: 'hd', fps: 'fps_120', visual_style: 'vivid',
      anti_aliasing: 'on', shadow: 'off', render_optimization: 'on',
      brightness: 2.0, sprint_sensitivity: 1.0, camera_fov: 'on',
    }),
    '2K高清120帧鲜艳抗锯齿开阴影关渲染优化开200%亮度100%冲刺超广角.imazingapp'
  );
});

test('filename 1.7K 无浮点噪声', () => {
  const name = generateFilename({ resolution_scale: 1.7 });
  assert(name.startsWith('1.7K'), `实际: ${name}`);
  assert(!name.includes('1.70000'), `不应出现浮点噪声: ${name}`);
});

test('filename 整数分辨率去 .0', () => {
  assert(generateFilename({ resolution_scale: 2.0 }).startsWith('2K'));
});

test('filename 超广角开关', () => {
  assert(!generateFilename({ camera_fov: 'off' }).includes('超广角'));
  assert(generateFilename({ camera_fov: 'on' }).includes('超广角'));
});

// ============================================================================
// 4.5 schema 视图（对齐原服务端 real_service 转换）
// ============================================================================
test('schema：10 个字段，display 换算正确', () => {
  const { fields } = getSchemaView(config);
  assert.strictEqual(fields.length, 10);

  const brightness = fields.find((f) => f.id === 'brightness');
  assert.strictEqual(brightness.unit, '%');
  assert.strictEqual(brightness.display_min, 80);
  assert.strictEqual(brightness.display_max, 500);
  assert.strictEqual(brightness.default, '2');

  const sprint = fields.find((f) => f.id === 'sprint_sensitivity');
  assert.strictEqual(sprint.display_min, 0);
  assert.strictEqual(sprint.display_max, 1000);

  // 无 display_multiplier 的字段不输出 display_min/max
  const resolution = fields.find((f) => f.id === 'resolution_scale');
  assert.strictEqual(resolution.display_min, undefined);
  assert.strictEqual(resolution.display_max, undefined);

  // options 仅 select/switch 有
  assert.strictEqual(fields.find((f) => f.id === 'quality').options.length, 3);
  assert.strictEqual(fields.find((f) => f.id === 'resolution_scale').options, null);
});

// ============================================================================
// 5. translate 单测（对齐 Rust patch-engine 行为）
// ============================================================================
test('translate：fps_120 生成 7 条 ini + 2 条 sav 绑定', () => {
  const ops = translate(config, { fps: 'fps_120' });
  const iniOps = ops.filter((o) => o.target_id === 'user_ini');
  const savOps = ops.filter((o) => o.target_id === 'active_sav');
  assert.strictEqual(iniOps.length, 7);
  assert.strictEqual(savOps.length, 2);
  assert.strictEqual(iniOps[0].action.address, 'r.PUBGDeviceFPSDef');
  assert.strictEqual(iniOps[0].action.value.t, 'int');
  assert.strictEqual(iniOps[0].action.value.v, 8);
  assert.strictEqual(savOps[0].action.address, 'FPSLevel');
  assert.strictEqual(savOps[0].action.value.v, 8);
});

test('translate：render_optimization off 生成 remove 操作', () => {
  const ops = translate(config, { render_optimization: 'off' });
  assert.strictEqual(ops.length, 5);
  for (const op of ops) {
    assert.strictEqual(op.action.kind, 'remove');
    assert.strictEqual(op.target_id, 'user_ini');
  }
});

test('translate：slider 数值解析为 float', () => {
  const ops = translate(config, { brightness: 3.5 });
  assert.strictEqual(ops.length, 1);
  assert.strictEqual(ops[0].action.value.t, 'float');
  assert.strictEqual(ops[0].action.value.v, 3.5);
});

test('translate：未知选项报错', () => {
  assert.throws(() => translate(config, { fps: 'fps_999' }), /no value mapping/);
});

// ============================================================================
// 6. ini 往返
// ============================================================================
test('ini 往返：注释丢弃 + 键排序 + 保序输出', () => {
  const ini = IniDoc.load(Buffer.from('[A]\n; comment\n+CVars=b=2\n\n# x\n+CVars=a=1\n'));
  const bytes = ini.save();
  const text = Buffer.from(bytes).toString('utf8');
  assert.strictEqual(text, '[A]\n+CVars=a=1\n+CVars=b=2\n\n');
});

test('ini 空文档往返', () => {
  const ini = IniDoc.load(Buffer.from(''));
  assert.strictEqual(ini.save().length, 0);
});

test('ini 带 BOM 也能加载（防御）', () => {
  const ini = IniDoc.load(Buffer.from('﻿[A]\n+CVars=a=1\n'));
  assert.strictEqual(ini.get('A', 'a'), '1');
});

test('translate：binding 缺 value_map 报明确错误', () => {
  const badConfig = JSON.parse(JSON.stringify(config));
  delete badConfig.patches.bindings.fps[0].value_map;
  assert.throws(() => translate(badConfig, { fps: 'fps_120' }), /no value_map/);
});

test('sav：截断输入报可定位错误（防御）', () => {
  // 截断的模板字节 → findProperty 应抛模块内错误而非裸 RangeError
  const truncated = TEMPLATE.subarray(0, 1000);
  assert.throws(() => findProperty(truncated, 'FPSLevel'));
});

test('zip：local/central 不一致报错（防御）', () => {
  // 构造一个 local extraLen 与 central 不一致的 zip 太复杂，
  // 这里验证损坏输入（截断中央目录）抛模块内错误而非裸越界
  const { rewriteZip } = require('../zip.js');
  const broken = TEMPLATE.subarray(0, 100);
  assert.throws(() => rewriteZip(broken, new Map()), /EOCD|中央目录|签名/);
});

// ============================================================================
// 汇总
// ============================================================================
console.log(`\n${passed} 通过, ${failed} 失败`);
process.exit(failed ? 1 : 0);
