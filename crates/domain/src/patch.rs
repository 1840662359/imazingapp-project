//! 补丁操作 —— Patch Engine 的输出、各写入引擎的输入。
//!
//! 一个 [`PatchOp`] 是"对某个目标地址执行某种操作"的原子指令，
//! 完全与业务无关。Patch Engine 不知道"fps"、"高清"等概念，
//! 只根据配置生成通用操作。

use serde::{Deserialize, Serialize};

use crate::value::TypedValue;

/// 单条与业务无关的补丁指令。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchOp {
    /// 目标 id（引用 targets.json 中的某个目标）。
    pub target_id: String,
    /// 操作类型。
    pub action: PatchAction,
}

/// 补丁操作类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatchAction {
    /// 设置值：对 ini 是"修改 cvar 的值"，对 sav 是"写属性"。
    Set {
        /// ini: cvar 名；sav: 属性名。
        address: String,
        /// 要写入的带类型的值。
        value: TypedValue,
    },

    /// 移除：对 ini 是"删除整行 +CVars=<cvar>=..."，对 sav 是"删除属性"。
    Remove {
        /// ini: cvar 名；sav: 属性名。
        address: String,
    },

    /// 追加：对 ini 是"在 section 末尾添加新行"，对 sav 是"新增属性"。
    /// （当前配置未用到，预留给将来扩展）
    Append {
        address: String,
        value: TypedValue,
    },

    /// 替换：查找匹配的旧值并替换为新值。
    /// （当前配置未用到，预留给复杂场景如正则替换）
    Replace {
        address: String,
        old_value: TypedValue,
        new_value: TypedValue,
    },

    /// 删除：与 Remove 语义类似，但可能用于数组元素删除等。
    /// （当前配置未用到，预留）
    Delete {
        address: String,
        /// 可选的索引/键（如删除数组第 N 项）。
        key: Option<String>,
    },
}

impl PatchOp {
    /// 创建一个 Set 操作。
    pub fn set(target_id: impl Into<String>, address: impl Into<String>, value: TypedValue) -> Self {
        Self {
            target_id: target_id.into(),
            action: PatchAction::Set {
                address: address.into(),
                value,
            },
        }
    }

    /// 创建一个 Remove 操作。
    pub fn remove(target_id: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            target_id: target_id.into(),
            action: PatchAction::Remove {
                address: address.into(),
            },
        }
    }

    /// 创建一个 Append 操作。
    pub fn append(target_id: impl Into<String>, address: impl Into<String>, value: TypedValue) -> Self {
        Self {
            target_id: target_id.into(),
            action: PatchAction::Append {
                address: address.into(),
                value,
            },
        }
    }
}
