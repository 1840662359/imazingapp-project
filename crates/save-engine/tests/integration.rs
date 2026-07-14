//! 集成测试：使用真实的 Active.sav 验证 SaveFile API。

use domain::value::TypedValue;
use save_engine::SaveFile;
use std::path::PathBuf;

fn get_real_sav_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("template.imazingapp")
}

#[test]
fn test_load_real_save() {
    // 从 imazingapp 中提取 Active.sav
    let zip_path = get_real_sav_path();
    assert!(zip_path.exists(), "模板文件不存在: {:?}", zip_path);

    let zip_file = std::fs::File::open(&zip_path).expect("打开 zip 失败");
    let mut archive = zip::ZipArchive::new(zip_file).expect("解析 zip 失败");

    let sav_path = "Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav";
    let mut sav_entry = archive.by_name(sav_path).expect("找不到 Active.sav");

    let mut sav_bytes = Vec::new();
    std::io::Read::read_to_end(&mut sav_entry, &mut sav_bytes).expect("读取 sav 失败");

    // 加载
    let save = SaveFile::load(&sav_bytes).expect("解析 sav 失败");

    // 验证属性数量
    assert!(save.len() > 0, "sav 应该有属性");
    println!("✓ 加载成功，共 {} 个属性", save.len());
}

#[test]
fn test_get_properties() {
    let zip_path = get_real_sav_path();
    let zip_file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let mut sav_entry = archive
        .by_name("Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav")
        .unwrap();
    let mut sav_bytes = Vec::new();
    std::io::Read::read_to_end(&mut sav_entry, &mut sav_bytes).unwrap();

    let save = SaveFile::load(&sav_bytes).expect("解析失败");

    // 测试已知属性（从之前的分析知道这些存在）
    let test_props = vec!["FPSLevel", "FpsLevel_TA", "ScreenLightness", "TpViewValue"];

    for prop_name in test_props {
        assert!(save.exists(prop_name), "属性 {} 应该存在", prop_name);

        let value = save.get(prop_name).expect(&format!("读取 {} 失败", prop_name));

        match value {
            TypedValue::Int(v) => println!("  {} = {} (Int)", prop_name, v),
            TypedValue::Float(v) => println!("  {} = {} (Float)", prop_name, v),
            TypedValue::Bool(v) => println!("  {} = {} (Bool)", prop_name, v),
            TypedValue::Str(v) => println!("  {} = {} (Str)", prop_name, v),
            TypedValue::Enum(v) => println!("  {} = {} (Enum)", prop_name, v),
        }
    }

    println!("✓ 所有已知属性读取成功");
}

#[test]
fn test_set_and_save() {
    let zip_path = get_real_sav_path();
    let zip_file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let mut sav_entry = archive
        .by_name("Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav")
        .unwrap();
    let mut sav_bytes = Vec::new();
    std::io::Read::read_to_end(&mut sav_entry, &mut sav_bytes).unwrap();

    let mut save = SaveFile::load(&sav_bytes).expect("解析失败");

    // 读取原始值
    let original_fps = save.get("FPSLevel").expect("读取 FPSLevel 失败");
    println!("原始 FPSLevel: {:?}", original_fps);

    // 修改
    save.set("FPSLevel", TypedValue::Int(7))
        .expect("设置 FPSLevel 失败");

    // 验证修改
    let new_fps = save.get("FPSLevel").expect("读取新 FPSLevel 失败");
    assert_eq!(new_fps, TypedValue::Int(7), "FPSLevel 应该被修改为 7");

    // 序列化
    let new_bytes = save.save().expect("序列化失败");
    assert!(new_bytes.len() > 0, "序列化后应有数据");

    // 重新加载验证
    let reloaded = SaveFile::load(&new_bytes).expect("重新加载失败");
    let reloaded_fps = reloaded.get("FPSLevel").expect("读取重新加载的 FPSLevel 失败");
    assert_eq!(reloaded_fps, TypedValue::Int(7), "重新加载后值应保持");

    println!("✓ 修改并序列化成功");
}

#[test]
fn test_remove_property() {
    let zip_path = get_real_sav_path();
    let zip_file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let mut sav_entry = archive
        .by_name("Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav")
        .unwrap();
    let mut sav_bytes = Vec::new();
    std::io::Read::read_to_end(&mut sav_entry, &mut sav_bytes).unwrap();

    let mut save = SaveFile::load(&sav_bytes).expect("解析失败");

    let original_len = save.len();
    assert!(save.exists("FPSLevel"), "FPSLevel 应该存在");

    // 删除
    save.remove("FPSLevel").expect("删除失败");

    assert_eq!(save.len(), original_len - 1, "属性数量应减少 1");
    assert!(!save.exists("FPSLevel"), "FPSLevel 应该被删除");

    // 序列化并重新加载
    let new_bytes = save.save().expect("序列化失败");
    let reloaded = SaveFile::load(&new_bytes).expect("重新加载失败");

    assert!(!reloaded.exists("FPSLevel"), "重新加载后 FPSLevel 应该不存在");
    assert_eq!(reloaded.len(), original_len - 1, "重新加载后属性数量应正确");

    println!("✓ 删除属性成功");
}

#[test]
#[ignore = "uesave 0.7 requires property schema for new properties"]
fn test_add_new_property() {
    // uesave 0.7 在序列化时需要每个属性的 schema 信息
    // 添加新属性会导致序列化失败，除非提供完整的 schema
    // 对于 imazingapp 项目，我们只需要修改已存在的属性，不需要添加新属性

    let zip_path = get_real_sav_path();
    let zip_file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let mut sav_entry = archive
        .by_name("Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav")
        .unwrap();
    let mut sav_bytes = Vec::new();
    std::io::Read::read_to_end(&mut sav_entry, &mut sav_bytes).unwrap();

    let mut save = SaveFile::load(&sav_bytes).expect("解析失败");

    let original_len = save.len();
    let new_prop_name = "TestNewProperty";

    assert!(!save.exists(new_prop_name), "新属性不应该存在");

    // 添加新属性
    save.set(new_prop_name, TypedValue::Int(999))
        .expect("添加新属性失败");

    assert_eq!(save.len(), original_len + 1, "属性数量应增加 1");
    assert!(save.exists(new_prop_name), "新属性应该存在");

    let value = save.get(new_prop_name).expect("读取新属性失败");
    assert_eq!(value, TypedValue::Int(999), "新属性值应正确");

    // 序列化会失败，因为 uesave 0.7 需要 property schema
    let result = save.save();
    assert!(result.is_err(), "序列化新属性应该失败（需要 schema）");

    println!("✓ 添加新属性成功（但无法序列化）");
}

#[test]
fn test_property_names() {
    let zip_path = get_real_sav_path();
    let zip_file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let mut sav_entry = archive
        .by_name("Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav")
        .unwrap();
    let mut sav_bytes = Vec::new();
    std::io::Read::read_to_end(&mut sav_entry, &mut sav_bytes).unwrap();

    let save = SaveFile::load(&sav_bytes).expect("解析失败");

    let names = save.property_names();
    assert!(names.len() > 0, "应该有属性名");
    assert!(names.contains(&"FPSLevel".to_string()), "应该包含 FPSLevel");

    println!("✓ 属性名列表正确，共 {} 个", names.len());
    println!("  前 10 个: {:?}", &names[..10.min(names.len())]);
}
