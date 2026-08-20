// miniprogram/engine/translate.js —— 用户选择 → 补丁操作序列。
//
// 对齐 Rust patch-engine 的 DefaultPatchTranslator：纯配置查表，零业务语义。
// 输入 selections: { fieldId: string(选项id) | number(连续值) }
// 输出 ops: [{ target_id, action: { kind:'set', address, value } | { kind:'remove', address } }]

'use strict';

const { toTypedValue, parseValueFromString } = require('./util.js');

/**
 * 将一组用户选择翻译为补丁操作序列。
 * @param {object} config 内嵌配置（config.js：schema/patches/targets/versions）
 * @param {object} selections { fieldId: string | number }
 * @returns {Array<{target_id, action}>}
 */
function translate(config, selections) {
  const ops = [];
  const bindings = config.patches.bindings;

  for (const [fieldId, selection] of Object.entries(selections)) {
    const fieldBindings = bindings[fieldId];
    if (!fieldBindings) {
      throw new Error(`field '${fieldId}' has no bindings in config`);
    }
    for (const binding of fieldBindings) {
      const address = binding.cvar || binding.property;
      if (!address) {
        throw new Error(`binding for field '${fieldId}' has neither cvar nor property`);
      }

      if (binding.remove_when && selection === binding.remove_when) {
        // 选中 remove_when 选项 → Remove 操作（ini 行删除）
        ops.push({ target_id: binding.target, action: { kind: 'remove', address } });
      } else {
        ops.push({ target_id: binding.target, action: { kind: 'set', address, value: genSetValue(fieldId, selection, binding) } });
      }
    }
  }
  return ops;
}

/** 生成 Set 操作的值（value_map 查表或用户输入解析）。 */
function genSetValue(fieldId, selection, binding) {
  if (binding.value_from_input) {
    // 连续型字段（slider）：直接取用户输入
    if (typeof selection === 'number') {
      return toTypedValue(selection, binding.value_type);
    }
    if (typeof selection === 'string') {
      // slider 值可能以字符串形式传来（预设展开等）
      return parseValueFromString(selection, binding.value_type);
    }
    throw new Error(`field '${fieldId}' expects a raw value, got ${JSON.stringify(selection)}`);
  }

  // 离散型字段（select/switch）：查 value_map
  if (typeof selection !== 'string') {
    throw new Error(`field '${fieldId}' expects an option id, got raw value`);
  }
  if (!binding.value_map) {
    throw new Error(`binding for field '${fieldId}' has no value_map`);
  }
  const mapped = binding.value_map[selection];
  if (mapped === undefined) {
    throw new Error(`field '${fieldId}' selected option '${selection}' has no value mapping`);
  }
  return toTypedValue(mapped, binding.value_type);
}

module.exports = { translate };
