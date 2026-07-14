//! Pipeline - 编排所有引擎生成配置文件
//!
//! ## 流程
//!
//! ```text
//! 用户选择 (Selections)
//!     ↓
//! Patch Engine → PatchOp[]
//!     ↓
//! 按 target_id 分组
//!     ↓
//! ┌─────────────┬─────────────┐
//! │ Active.sav  │ UserCustom.ini │
//! │ (Save Engine) │ (Ini Engine)   │
//! └─────────────┴─────────────┘
//!     ↓
//! Template Engine (流式转换)
//!     ↓
//! 和平精英.imazingapp (Vec<u8>)
//! ```
//!
//! ## 使用示例
//!
//! ```no_run
//! use pipeline::generate;
//! # use domain::error::Result;
//! # use config_compiler::CompiledConfig;
//! # use domain::selection::Selections;
//!
//! # fn example(config: &CompiledConfig, selections: &Selections, template_bytes: &[u8]) -> Result<Vec<u8>> {
//! let result = generate(&config, &selections, &template_bytes)?;
//! // result 是完整的 .imazingapp 文件字节流
//! # Ok(result)
//! # }
//! ```

use config_compiler::CompiledConfig;
use domain::error::{Error, Result};
use domain::patch::PatchOp;
use domain::selection::Selections;
use ini_engine::IniDoc;
use patch_engine::{DefaultPatchTranslator, PatchTranslator};
use save_engine::SaveFile;
use std::collections::HashMap;
use template_engine::TemplateEngine;

/// 单个 target 的字节转换器：输入原始文件字节，输出修改后的字节。
type Transformer = Box<dyn Fn(&[u8]) -> Result<Vec<u8>>>;

/// 生成配置文件
///
/// ## 参数
///
/// - `config`: 编译后的配置
/// - `selections`: 用户选择
/// - `template_bytes`: 模板文件字节流（和平精英.imazingapp）
///
/// ## 返回
///
/// 完整的 .imazingapp 文件字节流
///
/// ## 示例
///
/// ```no_run
/// # use pipeline::generate;
/// # use config_compiler::{CompiledConfig, load_and_compile};
/// # use domain::selection::Selections;
/// # use domain::error::Result;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<()> {
/// let config = load_and_compile(Path::new("config"))?;
/// let selections = Selections::new(); // 用户选择
/// let template = std::fs::read("和平精英.imazingapp")?;
///
/// let result = generate(&config, &selections, &template)?;
/// std::fs::write("output.imazingapp", result)?;
/// # Ok(())
/// # }
/// ```
pub fn generate(
    config: &CompiledConfig,
    selections: &Selections,
    template_bytes: &[u8],
) -> Result<Vec<u8>> {
    tracing::info!("开始生成配置文件");

    // 1. Patch Engine: 翻译用户选择为 PatchOp
    tracing::debug!("步骤 1: 翻译用户选择");
    let translator = DefaultPatchTranslator;
    let ops = translator.translate(config, selections)?;
    tracing::info!("生成了 {} 个 PatchOp", ops.len());

    // 2. 按 target_id 分组
    tracing::debug!("步骤 2: 按目标分组");
    let mut grouped: HashMap<String, Vec<&PatchOp>> = HashMap::new();
    for op in &ops {
        grouped.entry(op.target_id.clone()).or_default().push(op);
    }

    // 3. 构造 transformers
    tracing::debug!("步骤 3: 构造 transformers");
    let mut transformers: HashMap<String, Transformer> = HashMap::new();

    // 为每个 target 创建 transformer
    for (target_id, target_ops) in grouped {
        let target = config
            .targets
            .get(&target_id)
            .ok_or_else(|| Error::Validation(format!("未找到 target: {}", target_id)))?;

        let path = target.zip_path.clone();

        match target.kind {
            domain::target::TargetKind::Save => {
                // Active.sav - 使用 Save Engine
                tracing::debug!("为 {} 创建 Save Engine transformer", path);

                let ops_owned: Vec<PatchOp> = target_ops.iter().map(|op| (*op).clone()).collect();

                transformers.insert(
                    path,
                    Box::new(move |bytes: &[u8]| -> Result<Vec<u8>> {
                        tracing::debug!("处理 Active.sav，应用 {} 个 PatchOp", ops_owned.len());

                        let mut save = SaveFile::load(bytes)?;

                        for op in &ops_owned {
                            match &op.action {
                                domain::patch::PatchAction::Set { address, value } => {
                                    save.set(address, value.clone())?;
                                    tracing::trace!("set {} = {:?}", address, value);
                                }
                                domain::patch::PatchAction::Remove { address } => {
                                    save.remove(address)?;
                                    tracing::trace!("remove {}", address);
                                }
                                _ => {
                                    return Err(Error::Validation(
                                        "Save Engine 只支持 Set 和 Remove 操作".to_string(),
                                    ));
                                }
                            }
                        }

                        save.save()
                    }),
                );
            }
            domain::target::TargetKind::Ini => {
                // UserCustom.ini - 使用 Ini Engine
                tracing::debug!("为 {} 创建 Ini Engine transformer", path);

                let section = target
                    .section
                    .clone()
                    .unwrap_or_else(|| "[UserCustom DeviceProfile]".to_string());

                // 去掉 section 的方括号
                let section = section
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();

                let ops_owned: Vec<PatchOp> = target_ops.iter().map(|op| (*op).clone()).collect();

                transformers.insert(
                    path,
                    Box::new(move |bytes: &[u8]| -> Result<Vec<u8>> {
                        tracing::debug!("处理 UserCustom.ini，应用 {} 个 PatchOp", ops_owned.len());

                        let mut ini = IniDoc::load(bytes)?;

                        for op in &ops_owned {
                            match &op.action {
                                domain::patch::PatchAction::Set { address, value } => {
                                    // ini 的 address 格式：section:key 或直接 key
                                    let key = if address.contains(':') {
                                        address.split(':').nth(1).unwrap()
                                    } else {
                                        address.as_str()
                                    };

                                    // 将 TypedValue 转换为字符串
                                    let value_str = typed_value_to_string(value)?;

                                    ini.set(&section, key, &value_str)?;
                                    tracing::trace!("set {}:{} = {}", section, key, value_str);
                                }
                                domain::patch::PatchAction::Remove { address } => {
                                    let key = if address.contains(':') {
                                        address.split(':').nth(1).unwrap()
                                    } else {
                                        address.as_str()
                                    };

                                    ini.remove(&section, key)?;
                                    tracing::trace!("remove {}:{}", section, key);
                                }
                                _ => {
                                    return Err(Error::Validation(
                                        "Ini Engine 只支持 Set 和 Remove 操作".to_string(),
                                    ));
                                }
                            }
                        }

                        ini.save()
                    }),
                );
            }
        }
    }

    // 4. Template Engine: 流式转换
    tracing::debug!("步骤 4: 流式转换模板");
    let result = TemplateEngine::transform(template_bytes, transformers)?;

    tracing::info!("生成完成，输出大小: {} 字节", result.len());
    Ok(result)
}

/// 将 TypedValue 转换为字符串（用于 ini）
fn typed_value_to_string(value: &domain::value::TypedValue) -> Result<String> {
    match value {
        domain::value::TypedValue::Str(s) => Ok(s.clone()),
        domain::value::TypedValue::Int(i) => Ok(i.to_string()),
        domain::value::TypedValue::Float(f) => Ok(f.to_string()),
        domain::value::TypedValue::Bool(b) => Ok(if *b { "1" } else { "0" }.to_string()),
        domain::value::TypedValue::Enum(e) => Ok(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_value_to_string() {
        use domain::value::TypedValue;

        assert_eq!(
            typed_value_to_string(&TypedValue::Str("test".to_string())).unwrap(),
            "test"
        );
        assert_eq!(typed_value_to_string(&TypedValue::Int(42)).unwrap(), "42");
        assert_eq!(
            typed_value_to_string(&TypedValue::Float(2.5)).unwrap(),
            "2.5"
        );
        assert_eq!(typed_value_to_string(&TypedValue::Bool(true)).unwrap(), "1");
        assert_eq!(
            typed_value_to_string(&TypedValue::Bool(false)).unwrap(),
            "0"
        );
        assert_eq!(typed_value_to_string(&TypedValue::Enum(5)).unwrap(), "5");
    }
}
