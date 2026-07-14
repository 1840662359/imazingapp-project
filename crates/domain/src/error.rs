//! 统一错误类型。
//!
//! 各层通过 `?` 逐级上抛 [`Error`]，最终由 Interface 层（server）
//! 统一映射为 HTTP 响应。分类与架构分层一致。

use thiserror::Error;

/// 全项目统一 `Result` 别名。
pub type Result<T> = std::result::Result<T, Error>;

/// 全项目统一错误类型。
///
/// 按来源分类，便于 Interface 层做 4xx/5xx 映射与结构化日志。
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// 配置加载或编译失败（启动期 fail-fast）。
    #[error("config error: {0}")]
    Config(String),

    /// 用户输入 / 选择校验失败（对应 4xx）。
    #[error("validation error: {0}")]
    Validation(String),

    /// Save Engine（.sav 属性树）执行失败。
    #[error("save engine error: {0}")]
    Save(String),

    /// Ini Engine（明文 ini）执行失败。
    #[error("ini engine error: {0}")]
    Ini(String),

    /// Template Engine（流式模板转换）执行失败。
    #[error("template engine error: {0}")]
    Template(String),

    /// Zip Stream Engine（.imazingapp 拆包/重打包）执行失败。
    #[error("zip engine error: {0}")]
    Zip(String),

    /// 微信对接 / 广告回调验签失败。
    #[error("wechat error: {0}")]
    Wechat(String),

    /// 底层 IO 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化 / 反序列化错误。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// 未归类的内部错误。
    #[error("internal error: {0}")]
    Internal(String),
}
