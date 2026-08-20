//! golden —— 生成各配置组合的 golden fixtures。
//!
//! 输出到 out_dir：
//! - `case-NN.imazingapp`：pipeline 真实产物（JS 单测的对照基准）
//! - `case-NN.json`：{ name, selections, entries }，entries 为条目级
//!   清单（name / method / size / crc32），JS 侧逐条目对比用
//!
//! 注意：产物 zip 的时间戳不固定（Rust zip 默认当前时间），因此
//! 不做字节级对比，条目级等价即为验收标准。

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;

use config_compiler::{CompiledConfig, load_and_compile};
use domain::error::{Error, Result};
use domain::selection::{Selection, Selections};
use serde_json::{Value, json};

use crate::compression;

pub fn run(config_dir: &str, template_path: &str, out_dir: &str) -> Result<String> {
    let config = load_and_compile(Path::new(config_dir))?;
    let template = std::fs::read(template_path).map_err(Error::Io)?;

    std::fs::create_dir_all(out_dir).map_err(Error::Io)?;

    // 配置组合：01 全默认，02 全高配，03 渲染优化关（触发 remove_when），
    // 04 极值低配，05 精度回归（1.7K）+ 高亮高敏 + 超广角关
    let cases: Vec<(&str, Selections)> = vec![
        ("case-01", defaults(&config)?),
        (
            "case-02",
            from_pairs(&[
                ("quality", json!("hd")),
                ("fps", json!("fps_120")),
                ("resolution_scale", json!(2.0)),
                ("visual_style", json!("vivid")),
                ("anti_aliasing", json!("on")),
                ("shadow", json!("on")),
                ("render_optimization", json!("on")),
                ("brightness", json!(2.0)),
                ("camera_fov", json!("on")),
                ("sprint_sensitivity", json!(1.0)),
            ]),
        ),
        (
            "case-03",
            from_pairs(&[
                ("quality", json!("balanced")),
                ("fps", json!("fps_90")),
                ("resolution_scale", json!(1.5)),
                ("visual_style", json!("classic")),
                ("anti_aliasing", json!("off")),
                ("shadow", json!("off")),
                ("render_optimization", json!("off")),
                ("brightness", json!(1.5)),
                ("camera_fov", json!("off")),
                ("sprint_sensitivity", json!(0.5)),
            ]),
        ),
        (
            "case-04",
            from_pairs(&[
                ("quality", json!("smooth")),
                ("fps", json!("extreme")),
                ("resolution_scale", json!(1.0)),
                ("visual_style", json!("soft")),
                ("anti_aliasing", json!("off")),
                ("shadow", json!("on")),
                ("render_optimization", json!("on")),
                ("brightness", json!(0.8)),
                ("camera_fov", json!("on")),
                ("sprint_sensitivity", json!(0.0)),
            ]),
        ),
        (
            "case-05",
            from_pairs(&[
                ("quality", json!("hd")),
                ("fps", json!("fps_120")),
                ("resolution_scale", json!(1.7)),
                ("visual_style", json!("movie")),
                ("anti_aliasing", json!("on")),
                ("shadow", json!("on")),
                ("render_optimization", json!("on")),
                ("brightness", json!(5.0)),
                ("camera_fov", json!("off")),
                ("sprint_sensitivity", json!(10.0)),
            ]),
        ),
    ];

    let mut summary = String::new();
    for (label, selections) in &cases {
        let fname = format!("{label}.imazingapp");
        let fjson = format!("{label}.json");

        let output = pipeline::generate(&config, selections, &template)?;
        std::fs::write(Path::new(out_dir).join(&fname), &output).map_err(Error::Io)?;

        let entries = zip_entries(&output)?;
        let meta = json!({
            "name": label,
            "selections": selections_to_json(selections),
            "entries": entries,
        });
        std::fs::write(
            Path::new(out_dir).join(&fjson),
            serde_json::to_string_pretty(&meta).map_err(Error::Json)?,
        )
        .map_err(Error::Io)?;

        summary.push_str(&format!(
            "✓ {label}: {} 字节, {} 条目\n",
            output.len(),
            entries.len()
        ));
    }
    summary.push_str(&format!("共 {} 个 case，输出至 {}", cases.len(), out_dir));
    Ok(summary)
}

/// 从 schema 默认值构造选择集（case-01）。
fn defaults(config: &CompiledConfig) -> Result<Selections> {
    let mut s = Selections::new();
    for field in &config.schema.fields {
        let Some(def) = &field.default else { continue };
        let sel = if field.options.is_empty() {
            // 连续型字段（slider）：原样 Raw
            Selection::Raw(def.clone())
        } else {
            // 离散型字段（select/switch）：默认值应为选项 id 字符串
            match def.as_str() {
                Some(id) => Selection::Option(id.to_string()),
                None => {
                    return Err(Error::Config(format!(
                        "字段 {} 的 default 应为选项 id 字符串: {}",
                        field.id, def
                    )));
                }
            }
        };
        s.insert(field.id.clone(), sel);
    }
    Ok(s)
}

/// 从 (字段, JSON 值) 列表构造选择集：字符串 → Option，其它 → Raw。
fn from_pairs(pairs: &[(&str, Value)]) -> Selections {
    pairs
        .iter()
        .map(|(k, v)| match v {
            Value::String(id) => (k.to_string(), Selection::Option(id.clone())),
            _ => (k.to_string(), Selection::Raw(v.clone())),
        })
        .collect()
}

/// selections 序列化为简化格式：field_id → string | number。
fn selections_to_json(s: &Selections) -> Value {
    let map: BTreeMap<String, Value> = s
        .iter()
        .map(|(k, v)| match v {
            Selection::Option(id) => (k.clone(), json!(id)),
            Selection::Raw(r) => (k.clone(), r.clone()),
        })
        .collect();
    Value::Object(map.into_iter().collect())
}

/// 输出 zip 的条目级清单：name / method / size / crc32。
fn zip_entries(bytes: &[u8]) -> Result<Vec<Value>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| Error::Zip(format!("解析输出 zip 失败: {e}")))?;
    let mut list = Vec::new();
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| Error::Zip(format!("读取条目 {i} 失败: {e}")))?;
        list.push(json!({
            "name": entry.name(),
            "method": compression::code(entry.compression()),
            "size": entry.size(),
            "crc32": entry.crc32(),
        }));
    }
    Ok(list)
}
