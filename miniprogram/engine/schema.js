// miniprogram/engine/schema.js —— 内嵌配置 → 前端 schema 视图。
//
// 对齐原服务端 real_service.rs 的转换逻辑：
// - options：仅非空时输出
// - default：字符串化（字符串原样、数字 toString）
// - display_min/max：min/max × display_multiplier（三者齐全时）
// - unit：display_unit
// 纯函数模块，不依赖 wx API，可被 Node 直接测试。

'use strict';

/**
 * @param {object} config 内嵌配置（engine/config.js）
 * @returns {{ fields: Array, presets: Array }}
 */
function getSchemaView(config) {
  const fields = config.schema.fields.map((field) => {
    const options = field.options && field.options.length > 0
      ? field.options.map((opt) => ({ id: opt.id, label: opt.label }))
      : null;

    const hasAll = field.min !== undefined && field.max !== undefined && field.display_multiplier !== undefined;
    const displayMin = hasAll ? field.min * field.display_multiplier : undefined;
    const displayMax = hasAll ? field.max * field.display_multiplier : undefined;

    return {
      id: field.id,
      label: field.label || field.id,
      type: field.type,
      options,
      default: field.default !== undefined ? String(field.default) : undefined,
      min: field.min,
      max: field.max,
      step: field.step,
      unit: field.display_unit,
      display_min: displayMin,
      display_max: displayMax,
    };
  });

  return { fields, presets: [] };
}

module.exports = { getSchemaView };
