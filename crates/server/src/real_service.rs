//! Real Service 实现（生产环境使用）

use std::sync::Arc;

use async_trait::async_trait;
use config_compiler::CompiledConfig;
use domain::error::{Error, Result};
use domain::selection::Selections;

use crate::dto::{FieldSchema, OptionSchema, PresetDetail, SchemaResponse};
use crate::service::{BuildService, ConfigService};

/// Real 配置服务
///
/// 从编译后的 `CompiledConfig` 生成 schema
#[derive(Clone)]
pub struct RealConfigService {
    config: Arc<CompiledConfig>,
}

impl RealConfigService {
    pub fn new(config: Arc<CompiledConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ConfigService for RealConfigService {
    async fn get_schema(&self) -> Result<SchemaResponse> {
        let mut fields = Vec::new();

        // 从 schema 转换字段
        for field in &self.config.schema.fields {
            let options = if !field.options.is_empty() {
                Some(
                    field
                        .options
                        .iter()
                        .map(|opt| OptionSchema {
                            id: opt.id.clone(),
                            label: opt.label.clone(),
                        })
                        .collect(),
                )
            } else {
                None
            };

            // 计算 display_min 和 display_max
            let (display_min, display_max) = if let (Some(min), Some(max), Some(multiplier)) =
                (field.min, field.max, field.display_multiplier) {
                (Some(min * multiplier), Some(max * multiplier))
            } else {
                (None, None)
            };

            fields.push(FieldSchema {
                id: field.id.clone(),
                label: field.label.clone().unwrap_or_else(|| field.id.clone()),
                field_type: field.field_type.clone(),
                options,
                default: field.default.as_ref().map(|v| {
                    // 如果是字符串类型的 JSON 值，去掉引号
                    if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        // 数字等其他类型，直接转字符串
                        v.to_string()
                    }
                }),
                min: field.min,
                max: field.max,
                step: field.step,
                unit: field.display_unit.clone(),
                display_min,
                display_max,
            });
        }

        Ok(SchemaResponse {
            fields,
            presets: vec![], // 不再使用预设
        })
    }

    async fn get_presets(&self) -> Result<Vec<PresetDetail>> {
        // 不再使用预设功能
        Ok(vec![])
    }

    async fn expand_preset(&self, _preset_id: &str) -> Result<Selections> {
        Err(Error::Config("预设功能已禁用".to_string()))
    }
}

/// Real 构建服务
///
/// 调用 Pipeline 生成真实的配置文件
#[derive(Clone)]
pub struct RealBuildService {
    config: Arc<CompiledConfig>,
    template_bytes: Arc<Vec<u8>>,
}

impl RealBuildService {
    pub fn new(config: Arc<CompiledConfig>, template_bytes: Vec<u8>) -> Self {
        Self {
            config,
            template_bytes: Arc::new(template_bytes),
        }
    }
}

#[async_trait]
impl BuildService for RealBuildService {
    async fn build(&self, selections: &Selections) -> Result<Vec<u8>> {
        tracing::info!("构建 .imazingapp，选择数量: {}", selections.len());

        // 调用 Pipeline 生成
        let result = pipeline::generate(&self.config, selections, &self.template_bytes)?;

        tracing::info!("生成完成，输出大小: {} 字节", result.len());
        Ok(result)
    }
}
