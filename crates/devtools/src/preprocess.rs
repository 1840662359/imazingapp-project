//! preprocess —— 模板预处理：把目标条目（Active.sav / UserCustom.ini）改为 Stored 存储。
//!
//! 原理：完全复用 [`template_engine::TemplateEngine::transform`]——
//! 对目标条目挂 identity transformer（原字节原样返回），transform 内部会以
//! Stored 写回；其余条目走 raw_copy_file 原样保留压缩字节。
//!
//! 产物语义与生产一致：Rust pipeline 本来就把这两个条目以 Stored 写回。
//! 预处理后 JS 侧读取目标文件无需 inflate，zip 重写全程零压缩代码。

use std::collections::HashMap;
use std::fs;

use domain::error::Result;

use crate::compression;

/// 模板内目标文件路径（与 config/targets.json 保持一致）。
const SAV_PATH: &str = "Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav";
const INI_PATH: &str = "Container/Documents/ShadowTrackerExtra/Saved/Config/IOS/UserCustom.ini";

pub fn run(in_path: &str, out_path: &str) -> Result<()> {
    let input = fs::read(in_path).map_err(domain::Error::Io)?;
    println!("输入模板: {} ({} 字节)", in_path, input.len());

    // 目标条目挂 identity transformer：字节原样返回，Stored 写回
    let transformers: HashMap<String, template_engine::Transformer> = [
        (
            SAV_PATH.to_string(),
            Box::new(|bytes: &[u8]| Ok(bytes.to_vec())) as template_engine::Transformer,
        ),
        (
            INI_PATH.to_string(),
            Box::new(|bytes: &[u8]| Ok(bytes.to_vec())),
        ),
    ]
    .into_iter()
    .collect();

    let output = template_engine::TemplateEngine::transform(&input, transformers)?;
    fs::write(out_path, &output).map_err(domain::Error::Io)?;
    println!("输出模板: {} ({} 字节)", out_path, output.len());

    // 校验：目标条目确实已 Stored
    verify_stored(out_path)?;
    Ok(())
}

/// 校验预处理后的模板中，两个目标条目均为 Stored（method=0）。
fn verify_stored(template_path: &str) -> Result<()> {
    use std::io::Cursor;

    let bytes = fs::read(template_path).map_err(domain::Error::Io)?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| domain::Error::Zip(format!("打开模板失败: {e}")))?;

    for path in [SAV_PATH, INI_PATH] {
        let entry = archive
            .by_name(path)
            .map_err(|e| domain::Error::Zip(format!("条目不存在: {path}: {e}")))?;
        let method = compression::code(entry.compression());
        if method != 0 {
            return Err(domain::Error::Zip(format!(
                "条目 {path} 未 Stored 化（method={method}）"
            )));
        }
        println!("✓ {path} → Stored ({} 字节)", entry.size());
    }
    Ok(())
}
