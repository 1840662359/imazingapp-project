//! 集成测试：使用真实的 template.imazingapp 验证流式转换。

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use template_engine::{EntryPath, TemplateEngine, Transformer};

fn get_template_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("template.imazingapp")
}

#[test]
fn test_load_real_template() {
    let template_path = get_template_path();
    assert!(template_path.exists(), "模板文件不存在: {:?}", template_path);

    let template_bytes = std::fs::read(&template_path).expect("读取模板失败");
    assert!(template_bytes.len() > 0, "模板文件为空");

    println!("✓ 模板文件大小: {} bytes", template_bytes.len());
}

#[test]
fn test_transform_no_change() {
    let template_path = get_template_path();
    let template_bytes = std::fs::read(&template_path).unwrap();

    // 不做任何转换
    let transformers: HashMap<EntryPath, Transformer> = HashMap::new();
    let output_bytes = TemplateEngine::transform(&template_bytes, transformers).unwrap();

    // 验证输出是有效的 zip
    let output_cursor = std::io::Cursor::new(output_bytes);
    let mut output_zip = zip::ZipArchive::new(output_cursor).unwrap();

    // 验证 entry 数量相同
    let input_cursor = std::io::Cursor::new(&template_bytes);
    let input_zip = zip::ZipArchive::new(input_cursor).unwrap();
    assert_eq!(output_zip.len(), input_zip.len(), "entry 数量应相同");

    // 验证关键文件存在
    assert!(
        output_zip.by_name("Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav").is_ok(),
        "Active.sav 应该存在"
    );
    assert!(
        output_zip.by_name("Container/Documents/ShadowTrackerExtra/Saved/Config/IOS/UserCustom.ini").is_ok(),
        "UserCustom.ini 应该存在"
    );

    println!("✓ 无转换时输出与输入结构相同");
}

#[test]
fn test_transform_active_sav() {
    let template_path = get_template_path();
    let template_bytes = std::fs::read(&template_path).unwrap();

    let sav_path = "Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav";

    // 构造转换器：在 sav 开头添加标记
    let mut transformers: HashMap<EntryPath, Transformer> = HashMap::new();
    transformers.insert(
        sav_path.to_string(),
        Box::new(|bytes| {
            let mut new_bytes = b"TRANSFORMED:".to_vec();
            new_bytes.extend_from_slice(bytes);
            Ok(new_bytes)
        }),
    );

    let output_bytes = TemplateEngine::transform(&template_bytes, transformers).unwrap();

    // 验证转换生效
    let output_cursor = std::io::Cursor::new(output_bytes);
    let mut output_zip = zip::ZipArchive::new(output_cursor).unwrap();

    let transformed_bytes = {
        let mut transformed_entry = output_zip.by_name(sav_path).unwrap();
        let mut bytes = Vec::new();
        transformed_entry.read_to_end(&mut bytes).unwrap();
        bytes
    };

    assert!(
        transformed_bytes.starts_with(b"TRANSFORMED:"),
        "sav 应该被转换"
    );

    // 验证其它文件未被改动
    let ini_path = "Container/Documents/ShadowTrackerExtra/Saved/Config/IOS/UserCustom.ini";
    let ini_bytes = {
        let mut ini_entry = output_zip.by_name(ini_path).unwrap();
        let mut bytes = Vec::new();
        ini_entry.read_to_end(&mut bytes).unwrap();
        bytes
    };

    assert!(
        !ini_bytes.starts_with(b"TRANSFORMED:"),
        "ini 不应该被转换"
    );

    println!("✓ Active.sav 转换成功，其它文件未改动");
}

#[test]
fn test_transform_user_custom_ini() {
    let template_path = get_template_path();
    let template_bytes = std::fs::read(&template_path).unwrap();

    let ini_path = "Container/Documents/ShadowTrackerExtra/Saved/Config/IOS/UserCustom.ini";

    // 构造转换器：转为大写
    let mut transformers: HashMap<EntryPath, Transformer> = HashMap::new();
    transformers.insert(
        ini_path.to_string(),
        Box::new(|bytes| {
            let s = String::from_utf8_lossy(bytes);
            Ok(s.to_uppercase().into_bytes())
        }),
    );

    let output_bytes = TemplateEngine::transform(&template_bytes, transformers).unwrap();

    // 验证转换生效
    let output_cursor = std::io::Cursor::new(output_bytes);
    let mut output_zip = zip::ZipArchive::new(output_cursor).unwrap();

    let transformed_content = {
        let mut entry = output_zip.by_name(ini_path).unwrap();
        let mut content = String::new();
        entry.read_to_string(&mut content).unwrap();
        content
    };

    // 应该全是大写
    assert!(
        transformed_content.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_uppercase()),
        "ini 应该全部大写"
    );

    println!("✓ UserCustom.ini 转换成功");
}

#[test]
fn test_transform_both_targets() {
    let template_path = get_template_path();
    let template_bytes = std::fs::read(&template_path).unwrap();

    let sav_path = "Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav";
    let ini_path = "Container/Documents/ShadowTrackerExtra/Saved/Config/IOS/UserCustom.ini";

    // 两个转换器
    let mut transformers: HashMap<EntryPath, Transformer> = HashMap::new();
    transformers.insert(
        sav_path.to_string(),
        Box::new(|bytes| {
            let mut new_bytes = b"SAV_MARK:".to_vec();
            new_bytes.extend_from_slice(bytes);
            Ok(new_bytes)
        }),
    );
    transformers.insert(
        ini_path.to_string(),
        Box::new(|bytes| {
            let mut new_bytes = b"INI_MARK:".to_vec();
            new_bytes.extend_from_slice(bytes);
            Ok(new_bytes)
        }),
    );

    let output_bytes = TemplateEngine::transform(&template_bytes, transformers).unwrap();

    // 验证两个都被转换
    let output_cursor = std::io::Cursor::new(output_bytes);
    let mut output_zip = zip::ZipArchive::new(output_cursor).unwrap();

    let sav_bytes = {
        let mut entry = output_zip.by_name(sav_path).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        bytes
    };
    assert!(sav_bytes.starts_with(b"SAV_MARK:"), "sav 应该被转换");

    let ini_bytes = {
        let mut entry = output_zip.by_name(ini_path).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        bytes
    };
    assert!(ini_bytes.starts_with(b"INI_MARK:"), "ini 应该被转换");

    println!("✓ 两个目标文件同时转换成功");
}

#[test]
fn test_streaming_no_temp_files() {
    // 这个测试验证流式处理不会产生临时文件
    let template_path = get_template_path();
    let template_bytes = std::fs::read(&template_path).unwrap();

    let sav_path = "Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav";

    let mut transformers: HashMap<EntryPath, Transformer> = HashMap::new();
    transformers.insert(
        sav_path.to_string(),
        Box::new(|bytes| Ok(bytes.to_vec())), // 身份转换
    );

    // 在转换前后检查工作目录的文件数量
    let cwd = std::env::current_dir().unwrap();
    let before_count = std::fs::read_dir(&cwd).unwrap().count();

    let _output_bytes = TemplateEngine::transform(&template_bytes, transformers).unwrap();

    let after_count = std::fs::read_dir(&cwd).unwrap().count();

    assert_eq!(
        before_count, after_count,
        "不应该产生临时文件"
    );

    println!("✓ 流式处理无临时文件");
}

#[test]
fn test_output_is_valid_zip() {
    let template_path = get_template_path();
    let template_bytes = std::fs::read(&template_path).unwrap();

    let sav_path = "Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav";

    let mut transformers: HashMap<EntryPath, Transformer> = HashMap::new();
    transformers.insert(
        sav_path.to_string(),
        Box::new(|bytes| {
            // 做一个实际的修改
            let mut modified = bytes.to_vec();
            if !modified.is_empty() {
                modified[0] = modified[0].wrapping_add(1);
            }
            Ok(modified)
        }),
    );

    let output_bytes = TemplateEngine::transform(&template_bytes, transformers).unwrap();

    // 验证输出是合法的 zip
    let output_cursor = std::io::Cursor::new(&output_bytes);
    let output_zip = zip::ZipArchive::new(output_cursor).unwrap();

    assert!(output_zip.len() > 0, "输出 zip 应该有内容");

    // 可以写到临时文件并用外部工具验证（可选）
    // std::fs::write("/tmp/output.imazingapp", &output_bytes).unwrap();

    println!("✓ 输出是合法的 .imazingapp 文件");
}
