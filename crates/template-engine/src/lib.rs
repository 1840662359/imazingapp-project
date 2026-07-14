//! Template Engine —— 流式 .imazingapp 转换引擎。
//!
//! 采用 streaming 方式处理 zip：
//! - 读取输入 zip 的每个 entry
//! - 对 Active.sav / UserCustom.ini 调用 transformer
//! - 其它 entry 直接复制字节流到输出
//! - 不解压整个 zip，不生成临时目录
//!
//! 内存占用：O(单个 entry 大小)，不是 O(整个 zip)。

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use domain::error::{Error, Result};

/// Entry 路径。
pub type EntryPath = String;

/// Entry 内容（字节流）。
pub type EntryBytes = Vec<u8>;

/// Entry 转换器：旧字节 → 新字节。
///
/// 对 Active.sav / UserCustom.ini 等目标文件，Pipeline 提供转换逻辑。
pub type Transformer = Box<dyn Fn(&[u8]) -> Result<Vec<u8>>>;

/// 流式模板处理器。
///
/// 不持有任何状态，纯函数式。
pub struct TemplateEngine;

impl TemplateEngine {
    /// 流式转换 .imazingapp。
    ///
    /// # 参数
    /// - `input_bytes`: 输入的 .imazingapp 字节流
    /// - `transformers`: 路径 → 转换器的映射
    ///   - 键：zip 内的相对路径（如 `"Container/Documents/.../Active.sav"`）
    ///   - 值：转换函数
    ///
    /// # 流程
    /// 1. 打开输入 zip（只读）
    /// 2. 创建输出 zip（内存缓冲）
    /// 3. 遍历每个 entry：
    ///    - 如果路径在 transformers 里 → 读取 → 转换 → 写入输出
    ///    - 否则 → 直接复制字节流到输出（不解压到内存）
    /// 4. 返回输出 zip 字节流
    ///
    /// # Errors
    /// zip 读写失败、transformer 失败时返回错误。
    pub fn transform(
        input_bytes: &[u8],
        transformers: HashMap<EntryPath, Transformer>,
    ) -> Result<Vec<u8>> {
        // 输入 zip（只读）
        let input_cursor = Cursor::new(input_bytes);
        let mut input_zip = zip::ZipArchive::new(input_cursor)
            .map_err(|e| Error::Template(format!("open input zip failed: {}", e)))?;

        // 输出 zip（内存缓冲）
        let output_buf = Vec::new();
        let output_cursor = Cursor::new(output_buf);
        let mut output_zip = zip::ZipWriter::new(output_cursor);

        // 遍历每个 entry
        for i in 0..input_zip.len() {
            let mut entry = input_zip
                .by_index(i)
                .map_err(|e| Error::Template(format!("read entry {} failed: {}", i, e)))?;

            let entry_path = entry.name().to_string();
            let is_dir = entry.is_dir();
            let compression = entry.compression();

            // 目录条目直接复制
            if is_dir {
                let options = zip::write::FileOptions::<()>::default()
                    .compression_method(compression);
                output_zip
                    .add_directory(&entry_path, options)
                    .map_err(|e| Error::Template(format!("add directory {} failed: {}", entry_path, e)))?;
                continue;
            }

            // 文件条目
            let options = zip::write::FileOptions::<()>::default()
                .compression_method(compression)
                .unix_permissions(entry.unix_mode().unwrap_or(0o644));

            output_zip
                .start_file(&entry_path, options)
                .map_err(|e| Error::Template(format!("start file {} failed: {}", entry_path, e)))?;

            // 判断是否需要转换
            if let Some(transformer) = transformers.get(&entry_path) {
                // 读取 → 转换 → 写入
                let mut original_bytes = Vec::new();
                entry
                    .read_to_end(&mut original_bytes)
                    .map_err(|e| Error::Template(format!("read {} failed: {}", entry_path, e)))?;

                let transformed_bytes = transformer(&original_bytes)?;

                output_zip
                    .write_all(&transformed_bytes)
                    .map_err(|e| Error::Template(format!("write {} failed: {}", entry_path, e)))?;
            } else {
                // 直接复制字节流
                std::io::copy(&mut entry, &mut output_zip)
                    .map_err(|e| Error::Template(format!("copy {} failed: {}", entry_path, e)))?;
            }
        }

        // 完成输出 zip
        let output_cursor = output_zip
            .finish()
            .map_err(|e| Error::Template(format!("finish output zip failed: {}", e)))?;

        Ok(output_cursor.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_empty_transformers() {
        // 创建一个简单的 zip
        let input_buf = Vec::new();
        let input_cursor = Cursor::new(input_buf);
        let mut input_zip = zip::ZipWriter::new(input_cursor);

        let options = zip::write::FileOptions::<()>::default();
        input_zip.start_file("test.txt", options).unwrap();
        input_zip.write_all(b"hello").unwrap();

        let input_cursor = input_zip.finish().unwrap();
        let input_bytes = input_cursor.into_inner();

        // 不做任何转换
        let transformers = HashMap::new();
        let output_bytes = TemplateEngine::transform(&input_bytes, transformers).unwrap();

        // 验证输出
        let output_cursor = Cursor::new(output_bytes);
        let mut output_zip = zip::ZipArchive::new(output_cursor).unwrap();
        assert_eq!(output_zip.len(), 1);

        let mut entry = output_zip.by_name("test.txt").unwrap();
        let mut content = String::new();
        entry.read_to_string(&mut content).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_transform_with_transformer() {
        // 创建输入 zip
        let input_buf = Vec::new();
        let input_cursor = Cursor::new(input_buf);
        let mut input_zip = zip::ZipWriter::new(input_cursor);

        let options = zip::write::FileOptions::<()>::default();
        input_zip.start_file("file1.txt", options).unwrap();
        input_zip.write_all(b"original").unwrap();
        input_zip.start_file("file2.txt", options).unwrap();
        input_zip.write_all(b"untouched").unwrap();

        let input_cursor = input_zip.finish().unwrap();
        let input_bytes = input_cursor.into_inner();

        // 只转换 file1.txt
        let mut transformers: HashMap<EntryPath, Transformer> = HashMap::new();
        transformers.insert(
            "file1.txt".to_string(),
            Box::new(|bytes| {
                let s = String::from_utf8_lossy(bytes);
                Ok(s.to_uppercase().into_bytes())
            }),
        );

        let output_bytes = TemplateEngine::transform(&input_bytes, transformers).unwrap();

        // 验证输出
        let output_cursor = Cursor::new(output_bytes);
        let mut output_zip = zip::ZipArchive::new(output_cursor).unwrap();

        let content1 = {
            let mut file1 = output_zip.by_name("file1.txt").unwrap();
            let mut content = String::new();
            file1.read_to_string(&mut content).unwrap();
            content
        };
        assert_eq!(content1, "ORIGINAL");

        let content2 = {
            let mut file2 = output_zip.by_name("file2.txt").unwrap();
            let mut content = String::new();
            file2.read_to_string(&mut content).unwrap();
            content
        };
        assert_eq!(content2, "untouched");
    }

    #[test]
    fn test_transformer_error_propagates() {
        let input_buf = Vec::new();
        let input_cursor = Cursor::new(input_buf);
        let mut input_zip = zip::ZipWriter::new(input_cursor);

        let options = zip::write::FileOptions::<()>::default();
        input_zip.start_file("fail.txt", options).unwrap();
        input_zip.write_all(b"data").unwrap();

        let input_cursor = input_zip.finish().unwrap();
        let input_bytes = input_cursor.into_inner();

        let mut transformers: HashMap<EntryPath, Transformer> = HashMap::new();
        transformers.insert(
            "fail.txt".to_string(),
            Box::new(|_| Err(Error::Template("transformer error".to_string()))),
        );

        let result = TemplateEngine::transform(&input_bytes, transformers);
        assert!(result.is_err());
    }
}
