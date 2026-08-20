// miniprogram/engine/generate.js —— 纯前端生成编排（替代 Rust pipeline）。
//
// 流程：translate（选择→补丁）→ 按 target 分组 → sav/ini 原地修改
//       → zip 字节重写 → 输出 { bytes, filename }
//
// 纯函数模块：不依赖 wx API，输入模板字节与内嵌配置，输出新包字节。
// 模板必须为预处理版本（目标条目 Stored），由 devtools preprocess 产出。

'use strict';

const { translate } = require('./translate.js');
const { IniDoc } = require('./ini.js');
const { setValue, valueToBytes } = require('./sav.js');
const { readZipEntries, rewriteZip } = require('./zip.js');
const { generateFilename } = require('./filename.js');
const { typedValueToString } = require('./util.js');

/**
 * 生成 .imazingapp 字节。
 * @param {object} config 内嵌配置（engine/config.js）
 * @param {object} selections { fieldId: string | number }
 * @param {Uint8Array} templateBytes 预处理模板字节
 * @returns {{ bytes: Uint8Array, filename: string }}
 */
function generate(config, selections, templateBytes) {
  // 1. 翻译为补丁操作
  const ops = translate(config, selections);

  // 2. 按 target_id 分组
  const grouped = new Map(); // targetId → ops[]
  for (const op of ops) {
    if (!grouped.has(op.target_id)) grouped.set(op.target_id, []);
    grouped.get(op.target_id).push(op);
  }

  // 3. 逐 target 修改
  const mods = new Map();
  for (const [targetId, targetOps] of grouped) {
    const target = config.targets.targets[targetId];
    if (!target) throw new Error(`未找到 target: ${targetId}`);

    if (target.kind === 'save') {
      // Active.sav：定位 + 原地覆盖（仅支持 set）
      const savBytes = extractEntry(templateBytes, target.zip_path);
      let out = savBytes;
      for (const op of targetOps) {
        if (op.action.kind === 'set') {
          out = setValue(out, op.action.address, valueToBytes(op.action.value));
        } else {
          throw new Error('Save 目标仅支持 set 操作');
        }
      }
      mods.set(target.zip_path, out);
    } else if (target.kind === 'ini') {
      // UserCustom.ini：行级修改
      const iniBytes = extractEntry(templateBytes, target.zip_path);
      const section = (target.section || '[UserCustom DeviceProfile]')
        .replace(/^\[/, '')
        .replace(/\]$/, '');
      const ini = IniDoc.load(iniBytes);
      for (const op of targetOps) {
        if (op.action.kind === 'set') {
          ini.set(section, op.action.address, typedValueToString(op.action.value));
        } else if (op.action.kind === 'remove') {
          ini.remove(section, op.action.address);
        } else {
          throw new Error('Ini 目标仅支持 set/remove 操作');
        }
      }
      mods.set(target.zip_path, ini.save());
    } else {
      throw new Error(`未知 target kind: ${target.kind}`);
    }
  }

  // 4. zip 字节重写
  const bytes = rewriteZip(templateBytes, mods);
  const filename = generateFilename(selections);
  return { bytes, filename };
}

/** 从 zip 中提取条目字节（预处理模板中目标条目为 Stored，直接拷贝）。 */
function extractEntry(zipBytes, entryName) {
  const { entries } = readZipEntries(zipBytes);
  const entry = entries.find((e) => e.name === entryName);
  if (!entry) throw new Error(`zip: 条目不存在: ${entryName}`);
  if (entry.method !== 0) {
    throw new Error(`zip: 条目 ${entryName} 非 Stored（method=${entry.method}），请先运行 devtools preprocess`);
  }
  const dataStart = entry.localOffset + 30 + entry.nameLen + entry.extraLen;
  return zipBytes.subarray(dataStart, dataStart + entry.size);
}

module.exports = { generate };
