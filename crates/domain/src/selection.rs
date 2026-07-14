//! 用户选择 —— Patch Engine 的输入。
//!
//! 一次生成请求归约为一组 `字段 id → 选项 id`（或直接输入值）的选择。
//! 预设（preset）在 Pipeline 中先展开成同样的形态再进入流水线。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 单个字段上的用户选择。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Selection {
    /// 选中某个预定义选项（如 `fps_90`）。
    Option(String),
    /// 直接给出的原始输入值（用于连续型字段，如滑块）。
    Raw(serde_json::Value),
}

/// 一次请求的完整选择集合：字段 id → 选择。
///
/// 使用 `BTreeMap` 保证遍历顺序稳定，便于测试与日志复现。
pub type Selections = BTreeMap<String, Selection>;
