//! 原始配置模型 —— `config/` 下各 JSON 文件的 serde 反序列化结构。
//!
//! 本 crate 只描述配置的**语法结构**，不做语义校验（那是 config-compiler 的职责），
//! 也不含任何业务规则。所有字段与 JSON 一一对应。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use domain::target::TargetKind;
use domain::value::{TypedValue, ValueType};

/// `config/` 目录整体的原始视图（四个文件的聚合）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawConfig {
    pub versions: VersionsFile,
    pub targets: TargetsFile,
    pub schema: SchemaFile,
    pub patches: PatchesFile,
    pub presets: PresetsFile,
}

// ---- versions.json ----

/// 版本信息，用于配置 / 二进制 / 模板的兼容校验。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionsFile {
    /// 配置 schema 版本。
    pub config_schema_version: u32,
    /// 内置模板版本。
    pub template_version: String,
}

// ---- targets.json ----

/// 目标声明：模板内被修改文件的位置与解析器类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetsFile {
    /// 目标 id → 声明。
    pub targets: BTreeMap<String, TargetDecl>,
}

/// 单个目标的声明。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDecl {
    /// 目标种类（save / ini）。
    pub kind: TargetKind,
    /// 在 `.imazingapp`（zip）内的路径。
    pub zip_path: String,
    /// ini 目标的 section（可选，仅 kind=ini 时有效）。
    #[serde(default)]
    pub section: Option<String>,
}

// ---- schema.json ----

/// UI 唯一来源：字段、分组、选项、默认值、依赖。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFile {
    /// 表单字段列表。
    pub fields: Vec<FieldDef>,
}

/// 单个 UI 字段定义。只含语义 id 与展示文案，不含技术值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    /// 语义键，如 `fps`、`hdr`。
    pub id: String,
    /// UI 控件类型，如 `single_select`、`toggle`、`slider`。
    #[serde(rename = "type")]
    pub field_type: String,
    /// 展示标签（可选）。
    #[serde(default)]
    pub label: Option<String>,
    /// 分组，如 `画面`、`性能`。
    #[serde(default)]
    pub group: Option<String>,
    /// 可选项（单/多选类字段）。
    #[serde(default)]
    pub options: Vec<OptionDef>,
    /// 默认选项 id 或默认值。
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    /// slider 最小值（仅 slider 类型）。
    #[serde(default)]
    pub min: Option<f64>,
    /// slider 最大值（仅 slider 类型）。
    #[serde(default)]
    pub max: Option<f64>,
    /// slider 步进（仅 slider 类型）。
    #[serde(default)]
    pub step: Option<f64>,
    /// 显示单位（可选，如 "%"）。
    #[serde(default)]
    pub display_unit: Option<String>,
    /// 显示倍数（可选，如 100 表示显示为百分比）。
    #[serde(default)]
    pub display_multiplier: Option<f64>,
    /// 依赖：本字段可用的前置条件。
    #[serde(default)]
    pub depends_on: Vec<Dependency>,
}

/// 单个可选项。语义 id + 展示文案。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionDef {
    /// 语义键，如 `fps_90`。
    pub id: String,
    /// 展示文案（中文）。
    pub label: String,
}

/// 字段依赖声明（用于前端联动与配置校验）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// 依赖的字段 id。
    pub field: String,
    /// 该字段需取的选项 id 之一。
    pub any_of: Vec<String>,
}

// ---- patches.json ----

/// 业务映射唯一来源：字段 / 选项 → 目标操作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchesFile {
    /// 字段 id → 绑定列表。
    pub bindings: BTreeMap<String, Vec<Binding>>,
}

/// 单条绑定：描述某字段取值如何落到某目标地址。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// 目标 id（须在 targets.json 声明）。
    pub target: String,
    /// ini 目标的 cvar 名（当 target 是 ini 时使用，与 property 互斥）。
    #[serde(default)]
    pub cvar: Option<String>,
    /// sav 目标的属性名（当 target 是 save 时使用，与 cvar 互斥）。
    #[serde(default)]
    pub property: Option<String>,
    /// 值的逻辑类型。
    pub value_type: ValueType,
    /// 选项 id → 具体值（离散型字段）。
    #[serde(default)]
    pub value_map: BTreeMap<String, TypedValue>,
    /// 是否直接取用户输入值（连续型字段）。
    #[serde(default)]
    pub value_from_input: bool,
    /// 当选中该选项 id 时，移除此 ini 行而非写值（仅 ini 目标有效）。
    #[serde(default)]
    pub remove_when: Option<String>,
}

// ---- presets.json ----

/// 预设唯一来源：命名好的整套选择快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetsFile {
    /// 预设列表。
    pub presets: Vec<PresetDef>,
}

/// 单个预设。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetDef {
    /// 预设 id，如 `smooth`、`hd`。
    pub id: String,
    /// 展示文案。
    pub label: String,
    /// 字段 id → 选项 id 的选择快照。
    pub selections: BTreeMap<String, String>,
}
