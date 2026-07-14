//! 执行契约 —— 各引擎的抽象 trait。
//!
//! 上层（Pipeline）依赖这些抽象而非具体实现，从而可用假引擎做单元测试，
//! 也让 Save/Ini/Zip 的具体实现（含外部 crate）可独立替换。
//!
//! 注意：这些 trait 只引用领域类型，不含任何业务语义。

use std::collections::BTreeMap;

use crate::error::Result;
use crate::patch::PatchOp;

/// 内存中的 `.sav` 文档。由 [`SaveEngine::load`] 产出。
///
/// 每次请求应基于模板克隆一份，避免污染只读模板。
pub trait SaveDocument {
    /// 将单条补丁操作应用到属性树。类型不符时应返回错误，不做隐式转换。
    fn apply(&mut self, op: &PatchOp) -> Result<()>;

    /// 序列化回字节，满足 uesave 的字节级对齐要求。
    fn into_bytes(self) -> Result<Vec<u8>>;
}

/// `.sav` 引擎：字节 → 属性树。
pub trait SaveEngine {
    /// 关联的文档类型。
    type Document: SaveDocument;

    /// 从原始字节加载属性树。
    fn load(&self, bytes: &[u8]) -> Result<Self::Document>;
}

/// 内存中的 `.ini` 文档。由 [`IniEngine::load`] 产出。
///
/// 实现须保序、保留注释，仅改动被寻址的键。
pub trait IniDocument {
    /// 将单条补丁操作应用到 ini（按 section + key 定位）。
    fn apply(&mut self, op: &PatchOp) -> Result<()>;

    /// 序列化回明文字节。
    fn into_bytes(self) -> Result<Vec<u8>>;
}

/// `.ini` 引擎：字节 → 文档。
pub trait IniEngine {
    /// 关联的文档类型。
    type Document: IniDocument;

    /// 从明文字节加载文档。
    fn load(&self, bytes: &[u8]) -> Result<Self::Document>;
}

/// Zip Stream 引擎：拆解模板、替换指定条目、流式重打包。
///
/// 未被替换的条目应按字节复制，不重新压缩。
pub trait ZipEngine {
    /// 读取模板内某条目的解压字节（供写入引擎处理）。
    fn read_entry(&self, zip_path: &str) -> Result<Vec<u8>>;

    /// 用 `replacements`（zip 路径 → 新字节）替换对应条目后，
    /// 产出完整的新 `.imazingapp` 字节。其余条目字节复制。
    fn repack(&self, replacements: BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>>;
}
