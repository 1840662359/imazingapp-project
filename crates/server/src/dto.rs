//! API 数据传输对象 (DTOs)。
//!
//! 请求和响应的 JSON 结构定义。

use serde::{Deserialize, Serialize};

// ============================================================================
// GET /api/v1/schema
// ============================================================================

/// GET /api/v1/schema 响应
#[derive(Debug, Serialize)]
pub struct SchemaResponse {
    /// 字段列表
    pub fields: Vec<FieldSchema>,
    /// 预设列表
    pub presets: Vec<PresetMeta>,
}

/// 字段 schema
#[derive(Debug, Serialize)]
pub struct FieldSchema {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub options: Option<Vec<OptionSchema>>,
    pub default: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_max: Option<f64>,
}

/// 选项 schema
#[derive(Debug, Serialize)]
pub struct OptionSchema {
    pub id: String,
    pub label: String,
}

/// 预设元数据
#[derive(Debug, Serialize)]
pub struct PresetMeta {
    pub id: String,
    pub label: String,
}

// ============================================================================
// GET /api/v1/presets
// ============================================================================

/// GET /api/v1/presets 响应
#[derive(Debug, Serialize)]
pub struct PresetsResponse {
    pub presets: Vec<PresetDetail>,
}

/// 预设详情（包含选择）
#[derive(Debug, Serialize)]
pub struct PresetDetail {
    pub id: String,
    pub label: String,
    pub selections: std::collections::HashMap<String, String>,
}

// ============================================================================
// POST /api/v1/build
// ============================================================================

/// POST /api/v1/build 请求
#[derive(Debug, Deserialize)]
pub struct BuildRequest {
    /// 预设 ID（可选）
    pub preset_id: Option<String>,
    /// 自定义选择（可选）
    pub selections: Option<std::collections::HashMap<String, serde_json::Value>>,
}

// ============================================================================
// 错误响应
// ============================================================================

/// 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// ============================================================================
// 通用错误
// ============================================================================

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("validation_error", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message)
    }
}
