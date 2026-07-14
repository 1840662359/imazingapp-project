//! Patch Engine —— 纯函数式翻译器：用户选择 → 补丁操作序列。
//!
//! 输入 [`config_compiler::CompiledConfig`] + 已校验的 [`domain::Selections`]，
//! 输出 [`domain::PatchOp`] 列表。**完全不知道业务语义**（不知道 fps、高清等概念），
//! 只根据配置的绑定规则机械地生成 Set/Remove 等通用操作。
//!
//! 注意：本 crate 定义 [`PatchTranslator`] 契约（而非 domain），
//! 因为它依赖 `CompiledConfig`，放在 domain 会形成循环依赖。

use domain::error::{Error, Result};
use domain::patch::PatchOp;
use domain::selection::{Selection, Selections};
use domain::value::{TypedValue, ValueType};

use config_compiler::CompiledConfig;

/// 选择 → 补丁操作的翻译契约。
pub trait PatchTranslator {
    /// 将一组用户选择翻译为补丁操作序列。
    ///
    /// # Errors
    /// 选择非法、依赖不满足、同址值冲突时返回错误。
    fn translate(&self, config: &CompiledConfig, selections: &Selections) -> Result<Vec<PatchOp>>;
}

/// 默认翻译器实现：纯配置驱动，零业务逻辑。
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultPatchTranslator;

impl PatchTranslator for DefaultPatchTranslator {
    fn translate(&self, config: &CompiledConfig, selections: &Selections) -> Result<Vec<PatchOp>> {
        let mut ops = Vec::new();

        // 遍历所有选择，查配置的绑定，生成操作
        for (field_id, selection) in selections {
            // 查该字段的绑定列表
            let bindings = match config.bindings.get(field_id) {
                Some(b) => b,
                None => {
                    return Err(Error::Validation(format!(
                        "field '{}' has no bindings in config",
                        field_id
                    )))
                }
            };

            // 对每个绑定，生成一个操作
            for binding in bindings {
                // 决定操作类型和值
                let op = if let Some(remove_opt) = &binding.remove_when {
                    // 如果有 remove_when 且当前选中了该选项，生成 Remove
                    if matches!(selection, Selection::Option(opt) if opt == remove_opt) {
                        let address = binding
                            .ini_cvar
                            .clone()
                            .or_else(|| binding.sav_property.clone())
                            .ok_or_else(|| {
                                Error::Config(format!(
                                    "binding for field '{}' has neither ini_cvar nor sav_property",
                                    field_id
                                ))
                            })?;

                        PatchOp::remove(&binding.target_id, address)
                    } else {
                        // 否则是 Set 操作
                        generate_set_op(field_id, selection, binding)?
                    }
                } else {
                    // 没有 remove_when，就是普通 Set
                    generate_set_op(field_id, selection, binding)?
                };

                ops.push(op);
            }
        }

        // TODO: 校验同址冲突（两个操作写同一个 target_id + address）
        // TODO: 校验 depends_on

        Ok(ops)
    }
}

/// 生成一个 Set 操作（从绑定配置中查值或取用户输入）。
fn generate_set_op(
    field_id: &str,
    selection: &Selection,
    binding: &config_compiler::CompiledBinding,
) -> Result<PatchOp> {
    let address = binding
        .ini_cvar
        .clone()
        .or_else(|| binding.sav_property.clone())
        .ok_or_else(|| {
            Error::Config(format!(
                "binding for field '{}' has neither ini_cvar nor sav_property",
                field_id
            ))
        })?;

    let value = if binding.value_from_input {
        // 连续型字段（slider）：直接解析用户输入
        match selection {
            Selection::Raw(raw) => parse_value_from_json(raw, binding.value_type)?,
            Selection::Option(opt_str) => {
                // slider 也可能以字符串形式传来（如预设里的 "2.0"）
                parse_value_from_string(opt_str, binding.value_type)?
            }
        }
    } else {
        // 离散型字段（select/toggle）：查 value_map
        let opt_id = match selection {
            Selection::Option(id) => id,
            Selection::Raw(_) => {
                return Err(Error::Validation(format!(
                    "field '{}' expects an option id, got raw value",
                    field_id
                )))
            }
        };

        binding
            .value_map
            .get(opt_id)
            .cloned()
            .ok_or_else(|| {
                Error::Validation(format!(
                    "field '{}' selected option '{}' has no value mapping",
                    field_id, opt_id
                ))
            })?
    };

    Ok(PatchOp::set(&binding.target_id, address, value))
}

/// 将 JSON 值按类型解析为 TypedValue。
fn parse_value_from_json(json: &serde_json::Value, value_type: ValueType) -> Result<TypedValue> {
    match value_type {
        ValueType::Int => {
            let v = json.as_i64().ok_or_else(|| {
                Error::Validation(format!("expected int, got {:?}", json))
            })?;
            Ok(TypedValue::Int(v))
        }
        ValueType::Float => {
            let v = json.as_f64().ok_or_else(|| {
                Error::Validation(format!("expected float, got {:?}", json))
            })?;
            Ok(TypedValue::Float(v))
        }
        ValueType::Bool => {
            let v = json.as_bool().ok_or_else(|| {
                Error::Validation(format!("expected bool, got {:?}", json))
            })?;
            Ok(TypedValue::Bool(v))
        }
        ValueType::Str => {
            let v = json.as_str().ok_or_else(|| {
                Error::Validation(format!("expected string, got {:?}", json))
            })?;
            Ok(TypedValue::Str(v.to_string()))
        }
        ValueType::Enum => Err(Error::Validation(
            "enum type cannot use raw value".to_string(),
        )),
    }
}

/// 将用户输入的字符串按类型解析为 TypedValue。
fn parse_value_from_string(input: &str, value_type: ValueType) -> Result<TypedValue> {
    match value_type {
        ValueType::Int => {
            let v = input.parse::<i64>().map_err(|_| {
                Error::Validation(format!("cannot parse '{}' as int", input))
            })?;
            Ok(TypedValue::Int(v))
        }
        ValueType::Float => {
            let v = input.parse::<f64>().map_err(|_| {
                Error::Validation(format!("cannot parse '{}' as float", input))
            })?;
            Ok(TypedValue::Float(v))
        }
        ValueType::Bool => {
            let v = input.parse::<bool>().map_err(|_| {
                Error::Validation(format!("cannot parse '{}' as bool", input))
            })?;
            Ok(TypedValue::Bool(v))
        }
        ValueType::Str => Ok(TypedValue::Str(input.to_string())),
        ValueType::Enum => {
            // enum 不应该 value_from_input，因为需要 value_map
            Err(Error::Validation(
                "enum type cannot use value_from_input".to_string(),
            ))
        }
    }
}

/// 将预设展开为扁平选择（Pipeline 在翻译前调用）。
///
/// # Errors
/// 预设不存在时返回 [`domain::Error::Validation`]。
pub fn expand_preset(config: &CompiledConfig, preset_id: &str) -> Result<Selections> {
    let selections_map = config
        .preset_selections
        .get(preset_id)
        .ok_or_else(|| Error::Validation(format!("unknown preset: {}", preset_id)))?;

    // 将 BTreeMap<String, String> 转为 Selections (BTreeMap<String, Selection>)
    let selections = selections_map
        .iter()
        .map(|(k, v)| (k.clone(), Selection::Option(v.clone())))
        .collect();

    Ok(selections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::value::TypedValue;

    #[test]
    fn test_parse_value_from_string() {
        assert_eq!(
            parse_value_from_string("42", ValueType::Int).unwrap(),
            TypedValue::Int(42)
        );
        assert_eq!(
            parse_value_from_string("3.14", ValueType::Float).unwrap(),
            TypedValue::Float(3.14)
        );
        assert_eq!(
            parse_value_from_string("true", ValueType::Bool).unwrap(),
            TypedValue::Bool(true)
        );
        assert_eq!(
            parse_value_from_string("hello", ValueType::Str).unwrap(),
            TypedValue::Str("hello".to_string())
        );

        // 错误输入
        assert!(parse_value_from_string("not_a_number", ValueType::Int).is_err());
        assert!(parse_value_from_string("yes", ValueType::Bool).is_err());
    }

    #[test]
    fn test_parse_value_from_json() {
        use serde_json::json;

        assert_eq!(
            parse_value_from_json(&json!(42), ValueType::Int).unwrap(),
            TypedValue::Int(42)
        );
        assert_eq!(
            parse_value_from_json(&json!(3.14), ValueType::Float).unwrap(),
            TypedValue::Float(3.14)
        );
        assert_eq!(
            parse_value_from_json(&json!(true), ValueType::Bool).unwrap(),
            TypedValue::Bool(true)
        );
        assert_eq!(
            parse_value_from_json(&json!("hello"), ValueType::Str).unwrap(),
            TypedValue::Str("hello".to_string())
        );
    }
}

