//! 值类型系统 —— 业务与执行之间的唯一契约。
//!
//! 引擎不认识 "FPS" 或 "HDR"，只认识一个带类型的值该如何写入
//! .sav 属性树或 .ini 文本。所有配置里的技术值最终归约为 [`TypedValue`]。

use serde::{Deserialize, Serialize};

/// 逻辑值类型。决定该值在两种目标上的落地方式：
///
/// | 逻辑类型 | Save (uesave)              | Ini (文本)       |
/// |----------|----------------------------|------------------|
/// | `Int`    | Int / Int64Property        | `"3"`            |
/// | `Float`  | FloatProperty              | `"60.0"`         |
/// | `Bool`   | BoolProperty               | `"True"/"False"` |
/// | `Enum`   | Byte / EnumProperty        | `"2"`            |
/// | `Str`    | StrProperty                | `"High"`         |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    Int,
    Float,
    Bool,
    Enum,
    Str,
}

/// 携带逻辑类型的具体值。引擎据此构造对应的物理表示。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    /// 枚举底层以无符号字节表示。
    Enum(u8),
    Str(String),
}

impl TypedValue {
    /// 返回该值对应的逻辑类型。
    pub fn value_type(&self) -> ValueType {
        match self {
            TypedValue::Int(_) => ValueType::Int,
            TypedValue::Float(_) => ValueType::Float,
            TypedValue::Bool(_) => ValueType::Bool,
            TypedValue::Enum(_) => ValueType::Enum,
            TypedValue::Str(_) => ValueType::Str,
        }
    }
}
