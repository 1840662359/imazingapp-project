//! 集成测试：使用真实配置验证 Patch Engine 的翻译逻辑。

use std::collections::BTreeMap;
use std::path::PathBuf;

use config_compiler::load_and_compile;
use domain::patch::{PatchAction, PatchOp};
use domain::selection::{Selection, Selections};
use domain::value::TypedValue;
use patch_engine::{expand_preset, DefaultPatchTranslator, PatchTranslator};

fn load_real_config() -> std::sync::Arc<config_compiler::CompiledConfig> {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");

    load_and_compile(&config_dir).expect("配置加载失败")
}

#[test]
fn test_translate_simple_selection() {
    let config = load_real_config();
    let translator = DefaultPatchTranslator;

    // 构造一个简单选择：fps=120 + quality=流畅
    let mut selections = Selections::new();
    selections.insert("fps".to_string(), Selection::Option("fps_120".to_string()));
    selections.insert("quality".to_string(), Selection::Option("quality_smooth".to_string()));

    let ops = translator.translate(&config, &selections).expect("翻译失败");

    // fps 有 9 个绑定（7 个 ini + 2 个 sav）
    let fps_ops: Vec<_> = ops.iter().filter(|op| {
        matches!(&op.action, PatchAction::Set { address, value }
            if address.starts_with("r.PUBGDeviceFPS") || address == "FPSLevel" || address == "FpsLevel_TA")
    }).collect();
    assert_eq!(fps_ops.len(), 9, "fps 应生成 9 个操作");

    // 检查一个具体的操作
    let fps_def_op = ops.iter().find(|op| {
        matches!(&op.action, PatchAction::Set { address, .. } if address == "r.PUBGDeviceFPSDef")
    }).expect("找不到 r.PUBGDeviceFPSDef 操作");

    assert_eq!(fps_def_op.target_id, "user_ini");
    match &fps_def_op.action {
        PatchAction::Set { address, value } => {
            assert_eq!(address, "r.PUBGDeviceFPSDef");
            assert_eq!(value, &TypedValue::Int(8)); // 120帧 = 8
        }
        _ => panic!("期望 Set 操作"),
    }

    // quality 有 2 个绑定（r.UserQualitySetting + r.UserQualitySettingTA）
    let quality_ops: Vec<_> = ops.iter().filter(|op| {
        matches!(&op.action, PatchAction::Set { address, .. }
            if address.starts_with("r.UserQuality"))
    }).collect();
    assert_eq!(quality_ops.len(), 2, "quality 应生成 2 个操作");

    println!("✓ 简单选择翻译正确，生成 {} 个操作", ops.len());
}

#[test]
fn test_translate_with_remove_when() {
    let config = load_real_config();
    let translator = DefaultPatchTranslator;

    // render_bloom 关闭时应生成 Remove 操作
    let mut selections = Selections::new();
    selections.insert("render_bloom".to_string(), Selection::Option("bloom_off".to_string()));

    let ops = translator.translate(&config, &selections).expect("翻译失败");

    assert_eq!(ops.len(), 1);
    match &ops[0].action {
        PatchAction::Remove { address } => {
            assert_eq!(address, "r.BloomQuality");
            assert_eq!(ops[0].target_id, "user_ini");
        }
        _ => panic!("期望 Remove 操作，实际: {:?}", ops[0].action),
    }

    // render_bloom 开启时应生成 Set 操作
    let mut selections_on = Selections::new();
    selections_on.insert("render_bloom".to_string(), Selection::Option("bloom_on".to_string()));

    let ops_on = translator.translate(&config, &selections_on).expect("翻译失败");

    assert_eq!(ops_on.len(), 1);
    match &ops_on[0].action {
        PatchAction::Set { address, value } => {
            assert_eq!(address, "r.BloomQuality");
            assert_eq!(value, &TypedValue::Float(0.0));
        }
        _ => panic!("期望 Set 操作"),
    }

    println!("✓ remove_when 逻辑正确");
}

#[test]
fn test_translate_slider_value_from_input() {
    let config = load_real_config();
    let translator = DefaultPatchTranslator;

    // brightness 是 slider，value_from_input=true
    let mut selections = Selections::new();
    selections.insert("brightness".to_string(), Selection::Option("3.5".to_string()));

    let ops = translator.translate(&config, &selections).expect("翻译失败");

    assert_eq!(ops.len(), 1);
    match &ops[0].action {
        PatchAction::Set { address, value } => {
            assert_eq!(address, "ScreenLightness");
            assert_eq!(value, &TypedValue::Float(3.5));
            assert_eq!(ops[0].target_id, "active_sav");
        }
        _ => panic!("期望 Set 操作"),
    }

    println!("✓ slider value_from_input 正确解析");
}

#[test]
#[ignore = "预设功能已禁用"]
fn test_expand_preset() {
    let config = load_real_config();

    let selections = expand_preset(&config, "balanced").expect("预设展开失败");

    // balanced 预设应包含所有 14 个字段
    assert_eq!(selections.len(), 14, "预设字段数量不匹配");

    // 检查几个关键值
    assert_eq!(
        selections.get("fps"),
        Some(&Selection::Option("fps_90".to_string()))
    );
    assert_eq!(
        selections.get("quality"),
        Some(&Selection::Option("quality_balanced".to_string()))
    );
    assert_eq!(
        selections.get("shadow"),
        Some(&Selection::Option("shadow_on".to_string()))
    );

    println!("✓ 预设展开正确");
}

#[test]
#[ignore = "预设功能已禁用"]
fn test_translate_full_preset() {
    let config = load_real_config();
    let translator = DefaultPatchTranslator;

    // 展开 "smooth" 预设并翻译
    let selections = expand_preset(&config, "smooth").expect("预设展开失败");
    let ops = translator.translate(&config, &selections).expect("翻译失败");

    // smooth 预设关闭了 4 个渲染优化（应生成 4 个 Remove）+ 其它字段的 Set
    let remove_count = ops.iter().filter(|op| matches!(op.action, PatchAction::Remove { .. })).count();
    let set_count = ops.iter().filter(|op| matches!(op.action, PatchAction::Set { .. })).count();

    assert_eq!(remove_count, 4, "smooth 预设应生成 4 个 Remove 操作（bloom/lightshaft/tonemapper/depth off，shader on）");
    assert!(set_count > 0, "应有 Set 操作");

    println!("✓ 完整预设翻译正确：{} Remove + {} Set", remove_count, set_count);
}

#[test]
fn test_no_business_logic_in_engine() {
    let config = load_real_config();
    let translator = DefaultPatchTranslator;

    // 验证引擎完全不知道 "fps" 的语义
    // 只要配置里有绑定，任何 field_id 都能翻译
    let mut selections = Selections::new();
    selections.insert("fps".to_string(), Selection::Option("fps_90".to_string()));

    let ops = translator.translate(&config, &selections).expect("翻译失败");

    // 引擎只知道："fps" 这个 field_id 在配置里有 9 个绑定，生成 9 个操作
    // 至于 "fps" 是什么意思、为什么是 7，引擎完全不关心
    for op in &ops {
        match &op.action {
            PatchAction::Set { value, .. } => {
                // 90帧对应的值是 7，但引擎不知道这个映射，只从配置读
                if let TypedValue::Int(v) = value {
                    assert_eq!(*v, 7, "90帧应映射为 7（由配置决定）");
                }
            }
            _ => {}
        }
    }

    println!("✓ 引擎无业务逻辑，纯配置驱动");
}
