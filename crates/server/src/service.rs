//! Service 层接口定义。
//!
//! API 层通过这些 trait 调用业务逻辑，实现解耦。

use async_trait::async_trait;
use domain::error::Result;
use domain::selection::Selections;

use crate::dto::{PresetDetail, SchemaResponse};

/// 配置服务：提供 schema 和预设信息。
#[async_trait]
pub trait ConfigService: Send + Sync {
    /// 获取完整的 schema（字段 + 预设元数据）。
    async fn get_schema(&self) -> Result<SchemaResponse>;

    /// 获取所有预设的详细信息。
    async fn get_presets(&self) -> Result<Vec<PresetDetail>>;

    /// 展开预设为 Selections。
    async fn expand_preset(&self, preset_id: &str) -> Result<Selections>;
}

/// 构建服务：生成 .imazingapp 文件。
#[async_trait]
pub trait BuildService: Send + Sync {
    /// 构建 .imazingapp 文件。
    ///
    /// # 参数
    /// - `selections`: 用户选择
    ///
    /// # 返回
    /// 生成的 .imazingapp 字节流
    async fn build(&self, selections: &Selections) -> Result<Vec<u8>>;
}
