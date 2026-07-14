//! Zip Stream Engine —— `.imazingapp`（zip 容器）的拆包与流式重打包。
//!
//! 核心策略：仅重压被替换的条目，其余条目按字节复制。
//! 骨架阶段持有模板字节并实现 [`domain::engine::ZipEngine`] 契约，
//! 内部 zip 处理留待后续接入。不含业务语义。

use std::collections::BTreeMap;

use domain::engine::ZipEngine;
use domain::error::Result;

/// 基于内置模板的 Zip 引擎。模板常驻内存、只读。
#[derive(Debug, Clone)]
pub struct TemplateZipEngine {
    /// 模板 `.imazingapp` 的原始字节。
    template: Vec<u8>,
}

impl TemplateZipEngine {
    /// 用模板字节构造引擎。后续可在此预解析中央目录建立条目索引。
    pub fn new(template: Vec<u8>) -> Self {
        Self { template }
    }

    /// 返回模板原始字节大小（骨架期占位，避免字段未使用告警）。
    pub fn template_len(&self) -> usize {
        self.template.len()
    }
}

impl ZipEngine for TemplateZipEngine {
    fn read_entry(&self, _zip_path: &str) -> Result<Vec<u8>> {
        todo!("zip-engine: 解压并返回指定条目字节")
    }

    fn repack(&self, _replacements: BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
        todo!("zip-engine: 替换指定条目、其余字节复制，产出新包")
    }
}
