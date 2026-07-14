//! Config Compiler —— 把松散的原始配置编译成严格、索引化的运行时 IR。
//!
//! 职责（不含业务）：
//! 1. 语法校验（委托 config-model 的 serde 反序列化）。
//! 2. 跨文件语义校验：schema / patches / presets / targets 交叉引用一致，
//!    值类型匹配，依赖图无环，路径语法合法。
//! 3. 编译为不可变 [`CompiledConfig`]，供请求以 `Arc` 无锁只读共享。
//!
//! 校验失败即返回 [`domain::Error::Config`]，实现启动期 fail-fast。

use std::collections::BTreeMap;
use std::sync::Arc;

use domain::error::Result;
use domain::target::TargetKind;
use domain::value::{TypedValue, ValueType};

use config_model::RawConfig;

/// 编译后的、不可变的运行时配置。请求间以 `Arc` 共享。
#[derive(Debug, Clone)]
pub struct CompiledConfig {
    /// 供 UI 直接下发的 schema 视图（原样保留结构）。
    pub schema: config_model::SchemaFile,
    /// 供 UI 下发的预设列表。
    pub presets: Vec<config_model::PresetDef>,
    /// 预设 id → 展开后的字段选择（field_id → option_id）。
    pub preset_selections: BTreeMap<String, BTreeMap<String, String>>,
    /// 字段 id → 预编译绑定（运行时 O(1) 查表）。
    pub bindings: BTreeMap<String, Vec<CompiledBinding>>,
    /// 目标 id → 已解析的目标信息。
    pub targets: BTreeMap<String, CompiledTarget>,
    /// 版本信息。
    pub versions: config_model::VersionsFile,
}

/// 预编译的目标信息。
#[derive(Debug, Clone)]
pub struct CompiledTarget {
    pub kind: TargetKind,
    pub zip_path: String,
    /// ini 目标的 section（可选）。
    pub section: Option<String>,
}

/// 预编译的绑定：路径已解析，值已按类型编码。
#[derive(Debug, Clone)]
pub struct CompiledBinding {
    /// 目标 id。
    pub target_id: String,
    /// ini 的 cvar 名（与 sav_property 互斥）。
    pub ini_cvar: Option<String>,
    /// sav 的属性名（与 ini_cvar 互斥）。
    pub sav_property: Option<String>,
    /// 值逻辑类型。
    pub value_type: ValueType,
    /// 选项 id → 具体值（离散型）。
    pub value_map: BTreeMap<String, TypedValue>,
    /// 是否直接取用户输入（连续型）。
    pub value_from_input: bool,
    /// 当选中该选项 id 时，移除此 ini 行（仅 ini 有效）。
    pub remove_when: Option<String>,
}

/// 将原始配置编译为运行时 IR。
///
/// # Errors
/// 任一校验失败返回 [`domain::Error::Config`]。
pub fn compile(raw: RawConfig) -> Result<Arc<CompiledConfig>> {
    use domain::error::Error;
    use std::collections::{HashMap, HashSet};

    // 1. 构建字段/选项索引
    let mut field_ids = HashSet::new();
    let mut field_options: HashMap<String, HashSet<String>> = HashMap::new();

    for field in &raw.schema.fields {
        if !field_ids.insert(field.id.clone()) {
            return Err(Error::Config(format!("duplicate field id: {}", field.id)));
        }
        let mut opts = HashSet::new();
        for opt in &field.options {
            if !opts.insert(opt.id.clone()) {
                return Err(Error::Config(format!(
                    "duplicate option id '{}' in field '{}'",
                    opt.id, field.id
                )));
            }
        }
        field_options.insert(field.id.clone(), opts);
    }

    // 2. 校验 bindings 引用
    for (field_id, bindings) in &raw.patches.bindings {
        if !field_ids.contains(field_id) {
            return Err(Error::Config(format!(
                "patches.json references unknown field: {}",
                field_id
            )));
        }

        for binding in bindings {
            // 检查 target 存在
            if !raw.targets.targets.contains_key(&binding.target) {
                return Err(Error::Config(format!(
                    "binding for field '{}' references unknown target: {}",
                    field_id, binding.target
                )));
            }

            // 检查 cvar/property 互斥
            if binding.cvar.is_some() && binding.property.is_some() {
                return Err(Error::Config(format!(
                    "binding for field '{}' has both cvar and property",
                    field_id
                )));
            }
            if binding.cvar.is_none() && binding.property.is_none() {
                return Err(Error::Config(format!(
                    "binding for field '{}' has neither cvar nor property",
                    field_id
                )));
            }

            // 检查 value_map 的选项存在
            if let Some(opts) = field_options.get(field_id) {
                for opt_id in binding.value_map.keys() {
                    if !opts.contains(opt_id) {
                        return Err(Error::Config(format!(
                            "binding for field '{}' references unknown option: {}",
                            field_id, opt_id
                        )));
                    }
                }

                // 检查 remove_when 的选项存在
                if let Some(ref remove_opt) = binding.remove_when {
                    if !opts.contains(remove_opt) {
                        return Err(Error::Config(format!(
                            "binding for field '{}' has remove_when referencing unknown option: {}",
                            field_id, remove_opt
                        )));
                    }
                }
            }
        }
    }

    // 3. 校验 presets 引用
    for preset in &raw.presets.presets {
        for (field_id, opt_id) in &preset.selections {
            if !field_ids.contains(field_id) {
                return Err(Error::Config(format!(
                    "preset '{}' references unknown field: {}",
                    preset.id, field_id
                )));
            }
            // 只对有 options 的字段（single_select/toggle）验证选项存在性
            // slider 等连续型字段的值是字符串化的数字，不在 options 里
            if let Some(opts) = field_options.get(field_id) {
                if !opts.is_empty() && !opts.contains(opt_id) {
                    return Err(Error::Config(format!(
                        "preset '{}' references unknown option '{}' for field '{}'",
                        preset.id, opt_id, field_id
                    )));
                }
            }
        }
    }

    // 4. 构建编译后的目标
    let targets = raw
        .targets
        .targets
        .iter()
        .map(|(id, decl)| {
            (
                id.clone(),
                CompiledTarget {
                    kind: decl.kind,
                    zip_path: decl.zip_path.clone(),
                    section: decl.section.clone(),
                },
            )
        })
        .collect();

    // 5. 构建编译后的绑定
    let bindings = raw
        .patches
        .bindings
        .iter()
        .map(|(field_id, raw_bindings)| {
            let compiled: Vec<CompiledBinding> = raw_bindings
                .iter()
                .map(|b| CompiledBinding {
                    target_id: b.target.clone(),
                    ini_cvar: b.cvar.clone(),
                    sav_property: b.property.clone(),
                    value_type: b.value_type,
                    value_map: b.value_map.clone(),
                    value_from_input: b.value_from_input,
                    remove_when: b.remove_when.clone(),
                })
                .collect();
            (field_id.clone(), compiled)
        })
        .collect();

    // 6. 构建预设展开
    let preset_selections = raw
        .presets
        .presets
        .iter()
        .map(|p| (p.id.clone(), p.selections.clone()))
        .collect();

    Ok(Arc::new(CompiledConfig {
        schema: raw.schema,
        presets: raw.presets.presets,
        preset_selections,
        bindings,
        targets,
        versions: raw.versions,
    }))
}

/// 从 `config/` 目录加载并编译（读取四个 JSON 文件）。
///
/// # Errors
/// IO / 反序列化 / 语义校验失败均返回错误。
pub fn load_and_compile(config_dir: &std::path::Path) -> Result<Arc<CompiledConfig>> {
    use domain::error::Error;
    use std::fs;

    let read_json = |name: &str| -> Result<String> {
        let path = config_dir.join(name);
        fs::read_to_string(&path).map_err(|e| {
            Error::Config(format!("failed to read {}: {}", name, e))
        })
    };

    let versions: config_model::VersionsFile = serde_json::from_str(&read_json("versions.json")?)
        .map_err(|e| Error::Config(format!("versions.json parse error: {}", e)))?;

    let targets: config_model::TargetsFile = serde_json::from_str(&read_json("targets.json")?)
        .map_err(|e| Error::Config(format!("targets.json parse error: {}", e)))?;

    let schema: config_model::SchemaFile = serde_json::from_str(&read_json("schema.json")?)
        .map_err(|e| Error::Config(format!("schema.json parse error: {}", e)))?;

    let patches: config_model::PatchesFile = serde_json::from_str(&read_json("patches.json")?)
        .map_err(|e| Error::Config(format!("patches.json parse error: {}", e)))?;

    let presets: config_model::PresetsFile = serde_json::from_str(&read_json("presets.json")?)
        .map_err(|e| Error::Config(format!("presets.json parse error: {}", e)))?;

    let raw = RawConfig {
        versions,
        targets,
        schema,
        patches,
        presets,
    };

    compile(raw)
}
