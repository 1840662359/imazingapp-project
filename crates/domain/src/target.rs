//! 目标寻址 —— 描述"写到哪里"，与"写什么值"（[`crate::value`]）正交。

use serde::{Deserialize, Serialize};

/// 补丁目标种类。对应模板内两个被修改的文件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// `Active.sav` —— UE 二进制存档，经 uesave 解析。
    Save,
    /// `UserCustom.ini` —— 明文 ini。
    Ini,
}

/// 目标内部地址的单个路径段。
///
/// 由 Config Compiler 从配置里的路径字符串预解析而来，供引擎 O(1) 寻址。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathSegment {
    /// 具名字段 / 属性名，或 ini 的 section / key。
    Key(String),
    /// 数组下标，如 `Arr[2]`。
    Index(usize),
    /// Map 键，如 `Map["k"]`。
    MapKey(String),
}

/// 一个完整的目标地址：目标种类 + 有序路径段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetPath {
    /// 该地址属于哪个目标。
    pub kind: TargetKind,
    /// 预解析后的路径段序列。
    pub segments: Vec<PathSegment>,
}
