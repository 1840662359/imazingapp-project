//! HTTP 路由与处理器。
//!
//! 所有 handler 调用 Service trait，不直接实现业务逻辑。

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use tokio::sync::Semaphore;

/// 等待构建许可的最长时间。超时说明实例已饱和，返回 503 让客户端重试、
/// 平台按 CPU 使用率触发扩容，而不是无限排队把请求拖到超时。
const BUILD_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(20);

use domain::selection::{Selection, Selections};

use crate::dto::{BuildRequest, ErrorResponse, PresetsResponse, SchemaResponse};
use crate::service::{BuildService, ConfigService};

// ============================================================================
// 文件名生成
// ============================================================================

/// 根据用户选择生成文件名
///
/// 格式：<分辨率>K<画质><帧率><画面风格><屏幕亮度>亮度<冲刺灵敏度>冲刺<若视野大于80则加上超广角>
/// 例如：2K高清120帧鲜艳200%亮度100%冲刺超广角
fn generate_filename(selections: &Selections) -> String {
    let mut filename = String::new();

    // 1. 分辨率（resolution_scale: 1.0-2.0）
    if let Some(Selection::Raw(value)) = selections.get("resolution_scale") {
        if let Some(resolution) = value.as_f64() {
            // 保留小数：1.7 -> "1.7K"；整数则去掉 .0：2.0 -> "2K"。
            // 注意 f64 精度：1.7 实际存为 1.7000000000000002，
            // 步进为 0.1，用 {:.1} 保留一位小数即可精确显示、避免长串 0。
            if (resolution.fract()).abs() < f64::EPSILON {
                filename.push_str(&format!("{}K", resolution as i32));
            } else {
                filename.push_str(&format!("{:.1}K", resolution));
            }
        }
    }

    // 2. 画质（quality: smooth/balanced/hd）
    if let Some(Selection::Option(quality)) = selections.get("quality") {
        let quality_text = match quality.as_str() {
            "smooth" => "流畅",
            "balanced" => "均衡",
            "hd" => "高清",
            _ => "未知",
        };
        filename.push_str(quality_text);
    }

    // 3. 帧率（fps: extreme/fps_90/fps_120）
    if let Some(Selection::Option(fps)) = selections.get("fps") {
        let fps_text = match fps.as_str() {
            "extreme" => "60帧",
            "fps_90" => "90帧",
            "fps_120" => "120帧",
            _ => "未知帧",
        };
        filename.push_str(fps_text);
    }

    // 4. 画面风格（visual_style: classic/vivid/realistic/soft/movie）
    if let Some(Selection::Option(style)) = selections.get("visual_style") {
        let style_text = match style.as_str() {
            "classic" => "经典",
            "vivid" => "鲜艳",
            "realistic" => "写实",
            "soft" => "柔和",
            "movie" => "电影",
            _ => "未知风格",
        };
        filename.push_str(style_text);
    }

    // 4.1 抗锯齿开关（anti_aliasing: on/off）
    if let Some(Selection::Option(aa)) = selections.get("anti_aliasing") {
        filename.push_str(if aa == "on" {
            "抗锯齿开"
        } else {
            "抗锯齿关"
        });
    }

    // 4.2 阴影开关（shadow: on/off）
    if let Some(Selection::Option(shadow)) = selections.get("shadow") {
        filename.push_str(if shadow == "on" {
            "阴影开"
        } else {
            "阴影关"
        });
    }

    // 4.3 渲染优化开关（render_optimization: on/off）
    if let Some(Selection::Option(render_opt)) = selections.get("render_optimization") {
        filename.push_str(if render_opt == "on" {
            "渲染优化开"
        } else {
            "渲染优化关"
        });
    }

    // 5. 屏幕亮度（brightness: 0.8-5.0，显示为 80%-500%）
    if let Some(Selection::Raw(value)) = selections.get("brightness") {
        if let Some(brightness) = value.as_f64() {
            let brightness_percent = (brightness * 100.0).round() as i32;
            filename.push_str(&format!("{}%亮度", brightness_percent));
        }
    }

    // 6. 冲刺灵敏度（sprint_sensitivity: 0.0-10.0，显示为 0%-1000%）
    if let Some(Selection::Raw(value)) = selections.get("sprint_sensitivity") {
        if let Some(sensitivity) = value.as_f64() {
            let sensitivity_percent = (sensitivity * 100.0).round() as i32;
            filename.push_str(&format!("{}%冲刺", sensitivity_percent));
        }
    }

    // 7. 超广角（camera_fov > 80）
    if let Some(Selection::Raw(value)) = selections.get("camera_fov") {
        if let Some(fov) = value.as_f64() {
            if fov > 80.0 {
                filename.push_str("超广角");
            }
        }
    }

    // 添加扩展名
    filename.push_str(".imazingapp");

    filename
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub config_service: Arc<dyn ConfigService>,
    pub build_service: Arc<dyn BuildService>,
    /// 构建并发闸门：限制同时进行的 CPU 密集型 build 数量。
    /// 大小应与容器核数对齐，避免过多并发同时 deflate 导致 CPU 抖动与内存峰值。
    pub build_semaphore: Arc<Semaphore>,
}

/// 构建路由表
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/schema", get(get_schema))
        .route("/api/v1/presets", get(get_presets))
        .route("/api/v1/build", post(build))
        .with_state(state)
}

// ============================================================================
// Handlers
// ============================================================================

/// 健康检查
async fn health() -> &'static str {
    "ok"
}

/// GET /api/v1/schema
///
/// 返回配置 schema（字段 + 预设元数据）
async fn get_schema(State(state): State<AppState>) -> Result<Json<SchemaResponse>, AppError> {
    let schema = state.config_service.get_schema().await?;
    Ok(Json(schema))
}

/// GET /api/v1/presets
///
/// 返回所有预设的详细信息
async fn get_presets(State(state): State<AppState>) -> Result<Json<PresetsResponse>, AppError> {
    let presets = state.config_service.get_presets().await?;
    Ok(Json(PresetsResponse { presets }))
}

/// POST /api/v1/build
///
/// 构建 .imazingapp 文件，直接返回二进制内容（无存储层，天然支持高并发/多实例）。
/// 文件名通过 Content-Disposition 头返回。
async fn build(
    State(state): State<AppState>,
    Json(req): Json<BuildRequest>,
) -> Result<Response, AppError> {
    // 1. 解析输入：预设或自定义选择
    let selections = if let Some(preset_id) = req.preset_id {
        // 展开预设
        state.config_service.expand_preset(&preset_id).await?
    } else if let Some(raw_selections) = req.selections {
        // 解析自定义选择
        parse_selections(raw_selections)?
    } else {
        return Err(AppError::validation("必须提供 preset_id 或 selections"));
    };

    // 2. 获取构建许可（并发闸门）。等待超时说明实例已饱和，
    //    返回 503 让客户端稍后重试，同时高 CPU 触发平台扩容。
    let _permit = match tokio::time::timeout(
        BUILD_ACQUIRE_TIMEOUT,
        state.build_semaphore.clone().acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => permit,
        Ok(Err(_)) => return Err(AppError::internal("构建闸门已关闭")),
        Err(_) => return Err(AppError::unavailable("服务繁忙，请稍后重试")),
    };

    // 3. 调用构建服务（许可持有至本次请求结束）
    let bytes = state.build_service.build(&selections).await?;

    // 4. 生成文件名
    let filename = generate_filename(&selections);

    // 5. 直接返回二进制文件
    let content_disposition = format!("attachment; filename=\"{}\"", filename);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CONTENT_DISPOSITION, content_disposition.as_str()),
        ],
        bytes,
    )
        .into_response())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 解析前端发送的 selections
fn parse_selections(
    raw: std::collections::HashMap<String, serde_json::Value>,
) -> Result<Selections, AppError> {
    let mut selections = Selections::new();

    for (field_id, value) in raw {
        let selection = match value {
            serde_json::Value::String(s) => Selection::Option(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Selection::Raw(serde_json::Value::Number(i.into()))
                } else if let Some(f) = n.as_f64() {
                    Selection::Raw(serde_json::json!(f))
                } else {
                    return Err(AppError::validation(format!(
                        "字段 {} 的值类型不支持",
                        field_id
                    )));
                }
            }
            _ => {
                return Err(AppError::validation(format!(
                    "字段 {} 的值类型不支持",
                    field_id
                )));
            }
        };

        selections.insert(field_id, selection);
    }

    Ok(selections)
}

// ============================================================================
// 错误处理
// ============================================================================

/// API 错误包装
pub struct AppError {
    status: StatusCode,
    response: ErrorResponse,
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            response: ErrorResponse::validation_error(message),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            response: ErrorResponse::internal_error(message),
        }
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            response: ErrorResponse::new("service_unavailable", message),
        }
    }
}

impl From<domain::error::Error> for AppError {
    fn from(err: domain::error::Error) -> Self {
        use domain::error::Error;

        match err {
            Error::Validation(msg) => Self::validation(msg),
            Error::Config(msg) => Self::internal(format!("配置错误: {}", msg)),
            _ => Self::internal(err.to_string()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(self.response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 便捷构造：滑块类字段（原始数值）。
    fn raw(v: f64) -> Selection {
        Selection::Raw(serde_json::json!(v))
    }

    /// 便捷构造：选项类字段。
    fn opt(id: &str) -> Selection {
        Selection::Option(id.to_string())
    }

    #[test]
    fn filename_full_combo() {
        let mut s = Selections::new();
        s.insert("resolution_scale".into(), raw(2.0));
        s.insert("quality".into(), opt("hd"));
        s.insert("fps".into(), opt("fps_120"));
        s.insert("visual_style".into(), opt("vivid"));
        s.insert("anti_aliasing".into(), opt("on"));
        s.insert("shadow".into(), opt("off"));
        s.insert("render_optimization".into(), opt("on"));
        s.insert("brightness".into(), raw(2.0));
        s.insert("sprint_sensitivity".into(), raw(1.0));
        s.insert("camera_fov".into(), raw(90.0));

        assert_eq!(
            generate_filename(&s),
            "2K高清120帧鲜艳抗锯齿开阴影关渲染优化开200%亮度100%冲刺超广角.imazingapp"
        );
    }

    #[test]
    fn filename_resolution_decimal_no_float_noise() {
        // 回归：1.7 存为 f64 是 1.7000000000000002，必须显示为 1.7K
        let mut s = Selections::new();
        s.insert("resolution_scale".into(), raw(1.7));
        let name = generate_filename(&s);
        assert!(name.starts_with("1.7K"), "实际: {}", name);
        assert!(!name.contains("1.70000"), "不应出现浮点噪声: {}", name);
    }

    #[test]
    fn filename_resolution_integer_drops_decimal() {
        let mut s = Selections::new();
        s.insert("resolution_scale".into(), raw(2.0));
        assert!(generate_filename(&s).starts_with("2K"));
    }

    #[test]
    fn filename_fov_boundary() {
        // 视野 = 80 不加超广角；> 80 才加
        let mut at_80 = Selections::new();
        at_80.insert("camera_fov".into(), raw(80.0));
        assert!(!generate_filename(&at_80).contains("超广角"));

        let mut above_80 = Selections::new();
        above_80.insert("camera_fov".into(), raw(81.0));
        assert!(generate_filename(&above_80).contains("超广角"));
    }

    #[test]
    fn filename_switch_off_states() {
        let mut s = Selections::new();
        s.insert("anti_aliasing".into(), opt("off"));
        s.insert("shadow".into(), opt("off"));
        s.insert("render_optimization".into(), opt("off"));
        let name = generate_filename(&s);
        assert!(name.contains("抗锯齿关"));
        assert!(name.contains("阴影关"));
        assert!(name.contains("渲染优化关"));
    }

    #[test]
    fn filename_always_has_extension() {
        let s = Selections::new();
        assert_eq!(generate_filename(&s), ".imazingapp");
    }
}
