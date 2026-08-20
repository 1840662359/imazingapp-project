//! zip 压缩方法 → 数字编码（golden 清单 / 预处理校验共用）。
//!
//! zip 2.4 移除了旧 `Compression` 类型，仅剩 `CompressionMethod`，
//! 且 `to_u16` 已废弃，这里显式 match 保持无警告。

use zip::CompressionMethod;

/// 压缩方法编码：0 = Stored，8 = Deflated。
/// 模板只可能含这两种；其余方法防御性返回其原始编码。
#[allow(deprecated)] // zip 2.4 将 Unsupported variant 标记为 deprecated（用常量替代），此处需读取原始编码
pub fn code(method: CompressionMethod) -> u16 {
    match method {
        CompressionMethod::Stored => 0,
        CompressionMethod::Deflated => 8,
        CompressionMethod::Unsupported(v) => v,
        _ => 8,
    }
}
