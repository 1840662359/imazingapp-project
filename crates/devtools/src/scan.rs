//! scan —— 模板扫描断言。
//!
//! 从 config/ 动态收集所有绑定目标（零硬编码）：
//! - sav 目标：所有引用的属性必须存在于 Active.sav，且类型为定长（Int/Float = 4B）
//! - ini 目标：报告所有引用的 cvar；缺失项允许由生成引擎自动插入
//!
//! 模板升级（新游戏版本）时第一时间暴露属性缺失 / 改名 / 变长。
//! SAV 属性缺失仍会失败；INI cvar 缺失仅报告，不阻断生成。

use std::collections::BTreeSet;
use std::io::Cursor;
use std::path::Path;

use domain::error::{Error, Result};
use domain::target::TargetKind;
use domain::value::{TypedValue, ValueType};
use ini_engine::IniDoc;
use save_engine::SaveFile;

pub fn run(config_dir: &str, template_path: &str) -> Result<String> {
    let config = config_compiler::load_and_compile(Path::new(config_dir))?;
    let template_bytes = std::fs::read(template_path).map_err(Error::Io)?;

    // 打开模板 zip
    let mut archive = zip::ZipArchive::new(Cursor::new(template_bytes))
        .map_err(|e| Error::Zip(format!("打开模板失败: {e}")))?;

    let mut report = String::new();
    report.push_str(&format!("配置: {} 个字段\n", config.schema.fields.len()));
    report.push_str(&format!("模板: {} 个条目\n", archive.len()));

    // 按 target_id 收集绑定的地址
    let mut sav_props: BTreeSet<String> = BTreeSet::new();
    let mut ini_cvars: BTreeSet<String> = BTreeSet::new();
    for bindings in config.bindings.values() {
        for b in bindings {
            match config.targets.get(&b.target_id).map(|t| t.kind) {
                Some(TargetKind::Save) => {
                    if let Some(p) = &b.sav_property {
                        sav_props.insert(p.clone());
                    }
                }
                Some(TargetKind::Ini) => {
                    if let Some(c) = &b.ini_cvar {
                        ini_cvars.insert(c.clone());
                    }
                }
                None => {
                    return Err(Error::Config(format!("target 未声明: {}", b.target_id)));
                }
            }
        }
    }

    // ---- sav 断言 ----
    let sav_path = config
        .targets
        .get("active_sav")
        .map(|t| t.zip_path.clone())
        .ok_or_else(|| Error::Config("targets 缺少 active_sav".into()))?;

    let sav_bytes = read_entry(&mut archive, &sav_path)?;
    let save = SaveFile::load(&sav_bytes)?;
    report.push_str(&format!(
        "\n[Active.sav] {} 字节, {} 个属性\n",
        sav_bytes.len(),
        save.len()
    ));

    let mut saved_types: Vec<(String, ValueType)> = Vec::new();
    for prop in &sav_props {
        if !save.exists(prop) {
            return Err(Error::Config(format!(
                "Active.sav 缺少属性: {prop} —— 模板版本可能不兼容"
            )));
        }
        let value = save.get(prop)?;
        let vt = value.value_type();
        // 定长断言：原地覆盖要求 Int/Float（4 字节）
        if !matches!(vt, ValueType::Int | ValueType::Float) {
            return Err(Error::Config(format!(
                "属性 {prop} 类型为 {vt:?}，非定长（Int/Float），无法原地覆盖"
            )));
        }
        saved_types.push((prop.clone(), vt));
        report.push_str(&format!("✓ {prop}: {vt:?} = {}\n", fmt_value(&value)));
    }

    // ---- ini 断言 ----
    let ini_target = config
        .targets
        .get("user_ini")
        .ok_or_else(|| Error::Config("targets 缺少 user_ini".into()))?;
    let ini_bytes = read_entry(&mut archive, &ini_target.zip_path)?;
    let ini = IniDoc::load(&ini_bytes)?;
    let section = ini_target
        .section
        .clone()
        .unwrap_or_default()
        .trim_matches(['[', ']'])
        .to_string();
    report.push_str(&format!(
        "\n[UserCustom.ini] {} 字节, section: [{}]\n",
        ini_bytes.len(),
        section
    ));

    let mut missing: Vec<String> = Vec::new();
    for cvar in &ini_cvars {
        if ini.contains(&section, cvar) {
            report.push_str(&format!(
                "✓ {cvar} = {}\n",
                ini.get(&section, cvar).unwrap()
            ));
        } else {
            missing.push(cvar.clone());
            report.push_str(&format!("+ {cvar}: 模板中缺失，生成时自动插入\n"));
        }
    }

    // ---- 汇总 ----
    report.push_str(&format!(
        "\n汇总: {} 个 sav 属性（{} 定长）、{} 个 ini cvar（{} 已存在，{} 将自动插入）",
        saved_types.len(),
        saved_types.len(),
        ini_cvars.len(),
        ini_cvars.len() - missing.len(),
        missing.len()
    ));
    Ok(report)
}

fn read_entry(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>, path: &str) -> Result<Vec<u8>> {
    use std::io::Read;

    let mut entry = archive
        .by_name(path)
        .map_err(|e| Error::Zip(format!("模板内条目不存在: {path}: {e}")))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|e| Error::Zip(format!("读取 {path} 失败: {e}")))?;
    Ok(bytes)
}

fn fmt_value(v: &TypedValue) -> String {
    match v {
        TypedValue::Int(i) => i.to_string(),
        TypedValue::Float(f) => f.to_string(),
        TypedValue::Bool(b) => b.to_string(),
        TypedValue::Enum(e) => e.to_string(),
        TypedValue::Str(s) => s.clone(),
    }
}
