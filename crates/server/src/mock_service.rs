//! Mock Service 实现（用于 API 开发阶段）。

use async_trait::async_trait;
use domain::error::Result;
use domain::selection::Selections;

use crate::dto::{FieldSchema, OptionSchema, PresetDetail, SchemaResponse};
use crate::service::{BuildService, ConfigService};

/// Mock 配置服务
#[derive(Clone)]
pub struct MockConfigService;

#[async_trait]
impl ConfigService for MockConfigService {
    async fn get_schema(&self) -> Result<SchemaResponse> {
        Ok(SchemaResponse {
            fields: vec![
                // 1. 画质
                FieldSchema {
                    id: "quality".to_string(),
                    label: "画质".to_string(),
                    field_type: "select".to_string(),
                    options: Some(vec![
                        OptionSchema {
                            id: "smooth".to_string(),
                            label: "流畅".to_string(),
                        },
                        OptionSchema {
                            id: "balanced".to_string(),
                            label: "均衡".to_string(),
                        },
                        OptionSchema {
                            id: "hd".to_string(),
                            label: "高清".to_string(),
                        },
                    ]),
                    default: Some("balanced".to_string()),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    display_min: None,
                    display_max: None,
                },
                // 2. 帧率
                FieldSchema {
                    id: "fps".to_string(),
                    label: "帧率".to_string(),
                    field_type: "select".to_string(),
                    options: Some(vec![
                        OptionSchema {
                            id: "extreme".to_string(),
                            label: "极限".to_string(),
                        },
                        OptionSchema {
                            id: "fps_90".to_string(),
                            label: "90帧".to_string(),
                        },
                        OptionSchema {
                            id: "fps_120".to_string(),
                            label: "120帧".to_string(),
                        },
                    ]),
                    default: Some("fps_90".to_string()),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    display_min: None,
                    display_max: None,
                },
                // 3. 分辨率
                FieldSchema {
                    id: "resolution_scale".to_string(),
                    label: "分辨率".to_string(),
                    field_type: "slider".to_string(),
                    options: None,
                    default: Some("1.5".to_string()),
                    min: Some(1.0),
                    max: Some(2.0),
                    step: Some(0.1),
                    unit: None,
                    display_min: None,
                    display_max: None,
                },
                // 4. 画面风格
                FieldSchema {
                    id: "visual_style".to_string(),
                    label: "画面风格".to_string(),
                    field_type: "select".to_string(),
                    options: Some(vec![
                        OptionSchema {
                            id: "classic".to_string(),
                            label: "经典".to_string(),
                        },
                        OptionSchema {
                            id: "vivid".to_string(),
                            label: "鲜艳".to_string(),
                        },
                        OptionSchema {
                            id: "realistic".to_string(),
                            label: "写实".to_string(),
                        },
                        OptionSchema {
                            id: "soft".to_string(),
                            label: "柔和".to_string(),
                        },
                        OptionSchema {
                            id: "movie".to_string(),
                            label: "电影".to_string(),
                        },
                    ]),
                    default: Some("classic".to_string()),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    display_min: None,
                    display_max: None,
                },
                // 5. 抗锯齿
                FieldSchema {
                    id: "anti_aliasing".to_string(),
                    label: "抗锯齿".to_string(),
                    field_type: "switch".to_string(),
                    options: None,
                    default: Some("true".to_string()),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    display_min: None,
                    display_max: None,
                },
                // 6. 阴影
                FieldSchema {
                    id: "shadow".to_string(),
                    label: "阴影".to_string(),
                    field_type: "switch".to_string(),
                    options: None,
                    default: Some("true".to_string()),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    display_min: None,
                    display_max: None,
                },
                // 7. 屏幕亮度
                FieldSchema {
                    id: "brightness".to_string(),
                    label: "屏幕亮度 (80%-500%)".to_string(),
                    field_type: "slider".to_string(),
                    options: None,
                    default: Some("1.0".to_string()),
                    min: Some(0.8),
                    max: Some(5.0),
                    step: Some(0.1),
                    unit: Some("%".to_string()),
                    display_min: Some(80.0),
                    display_max: Some(500.0),
                },
                // 8. 第三人称镜头视野
                FieldSchema {
                    id: "camera_fov".to_string(),
                    label: "第三人称镜头视野".to_string(),
                    field_type: "slider".to_string(),
                    options: None,
                    default: Some("85".to_string()),
                    min: Some(80.0),
                    max: Some(90.0),
                    step: Some(1.0),
                    unit: None,
                    display_min: None,
                    display_max: None,
                },
                // 9. 冲刺灵敏度
                FieldSchema {
                    id: "sprint_sensitivity".to_string(),
                    label: "冲刺灵敏度 (0%-1000%)".to_string(),
                    field_type: "slider".to_string(),
                    options: None,
                    default: Some("1.0".to_string()),
                    min: Some(0.0),
                    max: Some(10.0),
                    step: Some(0.1),
                    unit: Some("%".to_string()),
                    display_min: Some(0.0),
                    display_max: Some(1000.0),
                },
            ],
            presets: vec![], // 不再使用预设
        })
    }

    async fn get_presets(&self) -> Result<Vec<PresetDetail>> {
        let mut selections = std::collections::HashMap::new();
        selections.insert("fps".to_string(), "fps_60".to_string());
        selections.insert("brightness".to_string(), "2.0".to_string());

        Ok(vec![PresetDetail {
            id: "smooth".to_string(),
            label: "流畅（省电）".to_string(),
            selections,
        }])
    }

    async fn expand_preset(&self, preset_id: &str) -> Result<Selections> {
        let mut selections = Selections::new();
        selections.insert(
            "fps".to_string(),
            domain::selection::Selection::Option("fps_60".to_string()),
        );

        tracing::info!("展开预设: {}", preset_id);
        Ok(selections)
    }
}

/// Mock 构建服务
#[derive(Clone)]
pub struct MockBuildService;

#[async_trait]
impl BuildService for MockBuildService {
    async fn build(&self, selections: &Selections) -> Result<Vec<u8>> {
        tracing::info!("构建 .imazingapp，选择数量: {}", selections.len());

        // 返回假数据（模拟 1MB 文件）
        Ok(vec![0u8; 1024 * 1024])
    }
}
