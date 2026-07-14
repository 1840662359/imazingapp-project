//! 领域层：全项目共享的类型、错误与执行契约（trait）。
//!
//! 本 crate 是依赖图的叶子，不含任何业务语义，也不依赖其它内部 crate。
//! 它定义三类东西：
//!
//! 1. [`error`] —— 统一错误类型与 [`Result`] 别名。
//! 2. 公共 Model —— 值类型系统（[`value`]）、目标寻址（[`target`]）、
//!    补丁操作（[`patch`]）、用户选择（[`selection`]）。
//! 3. 执行契约 —— 各引擎的抽象 trait（[`engine`]），供上层依赖抽象而非实现。

pub mod engine;
pub mod error;
pub mod patch;
pub mod selection;
pub mod target;
pub mod value;

pub use error::{Error, Result};
pub use patch::PatchOp;
pub use selection::{Selection, Selections};
pub use target::{PathSegment, TargetKind, TargetPath};
pub use value::{TypedValue, ValueType};
