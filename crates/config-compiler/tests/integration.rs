//! 集成测试：加载真实配置并验证编译成功。

use config_compiler::load_and_compile;
use std::path::PathBuf;

#[test]
fn load_real_config() {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");

    let result = load_and_compile(&config_dir);
    assert!(result.is_ok(), "配置加载失败: {:?}", result.err());

    let config = result.unwrap();

    // 验证基本结构
    assert_eq!(config.versions.config_schema_version, 1);
    assert_eq!(config.versions.template_version, "1.0.0");

    // 验证字段数量
    assert_eq!(config.schema.fields.len(), 14, "字段数量不匹配");

    // 验证目标
    assert_eq!(config.targets.len(), 2, "目标数量不匹配");
    assert!(config.targets.contains_key("user_ini"));
    assert!(config.targets.contains_key("active_sav"));

    // 验证 ini 目标有 section
    let ini_target = &config.targets["user_ini"];
    assert_eq!(
        ini_target.section.as_deref(),
        Some("[UserCustom DeviceProfile]")
    );

    // 验证预设（已禁用）
    assert_eq!(config.presets.len(), 0, "预设数量不匹配");

    // 验证关键字段的绑定
    assert!(config.bindings.contains_key("fps"));
    assert!(config.bindings.contains_key("quality"));
    assert!(config.bindings.contains_key("brightness"));

    // 验证 fps 有绑定
    assert!(!config.bindings["fps"].is_empty(), "fps 应该有绑定");

    println!("✓ 配置加载成功，所有验证通过");
}

#[test]
fn validate_field_option_refs() {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config");

    let config = load_and_compile(&config_dir).expect("配置加载失败");

    // 验证每个绑定的 value_map 都引用了有效的选项
    for (field_id, bindings) in &config.bindings {
        let field = config
            .schema
            .fields
            .iter()
            .find(|f| &f.id == field_id)
            .expect(&format!("字段 {} 不存在", field_id));

        let valid_opts: std::collections::HashSet<_> =
            field.options.iter().map(|o| o.id.as_str()).collect();

        for binding in bindings {
            for opt_id in binding.value_map.keys() {
                assert!(
                    valid_opts.contains(opt_id.as_str()),
                    "字段 {} 的绑定引用了无效选项: {}",
                    field_id,
                    opt_id
                );
            }
        }
    }

    println!("✓ 所有绑定的选项引用都有效");
}
