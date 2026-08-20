//! dump —— 打印 uesave 解析出的 sav 属性结构（开发期诊断用）。

use std::io::Cursor;

use domain::error::{Error, Result};
use save_engine::SaveFile;

pub fn run(template_path: &str) -> Result<String> {
    let template_bytes = std::fs::read(template_path).map_err(domain::Error::Io)?;
    let mut archive = zip::ZipArchive::new(Cursor::new(template_bytes))
        .map_err(|e| Error::Zip(format!("打开模板失败: {e}")))?;

    use std::io::Read;
    let mut entry = archive
        .by_name("Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav")
        .map_err(|e| Error::Zip(format!("条目不存在: {e}")))?;
    let mut sav_bytes = Vec::new();
    entry.read_to_end(&mut sav_bytes).map_err(Error::Io)?;

    let save = SaveFile::load(&sav_bytes)?;

    let mut out = String::new();
    out.push_str(&format!("属性数量: {}\n", save.len()));
    for (i, name) in save.property_names().iter().enumerate() {
        match save.get(name) {
            Ok(v) => out.push_str(&format!("[{i:3}] {name}: {v:?}\n")),
            Err(e) => out.push_str(&format!("[{i:3}] {name}: <err: {e}>\n")),
        }
    }

    // ---- 实验：字节级扫描验证（sav.js 原型）----
    if std::env::args().any(|a| a == "--scan") {
        out.push_str("\n--- 字节级扫描（sav.js 原型）---\n");
        out.push_str(&scan_properties(&sav_bytes, &save)?);
    }

    // ---- 实验：修改 FPSLevel 与 LeftRightShoot 后的字节级 diff ----
    if std::env::args().any(|a| a == "--diff") {
        let mut save2 = SaveFile::load(&sav_bytes)?;
        save2.set("FPSLevel", domain::value::TypedValue::Int(6))?;
        save2.set("LeftRightShoot", domain::value::TypedValue::Bool(false))?;
        let new_bytes = save2.save()?;

        out.push_str(&format!(
            "\n--- diff 实验 ---\n修改 FPSLevel→6, LeftRightShoot→false\n新旧字节数: {} → {}\n",
            sav_bytes.len(),
            new_bytes.len()
        ));
        let n = sav_bytes.len().min(new_bytes.len());
        for i in 0..n {
            if sav_bytes[i] != new_bytes[i] {
                out.push_str(&format!(
                    "不同 @ 0x{i:04x} ({}): {:02x} → {:02x}\n",
                    i, sav_bytes[i], new_bytes[i]
                ));
            }
        }
    }
    Ok(out)
}

/// sav.js 原型：按 name+type+size+index+data+值区 扫描属性序列，
/// 用 uesave 的 dump 值对照验证定位正确性。
fn scan_properties(sav_bytes: &[u8], save: &SaveFile) -> Result<String> {
    use std::collections::BTreeMap;

    // ---- FString 读取（len 含 NUL）----
    fn read_fstring(bytes: &[u8], off: usize) -> Result<(String, usize)> {
        if off + 4 > bytes.len() {
            return Err(Error::Save(format!("FString 越界 @ 0x{off:x}")));
        }
        let len = i32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        if len < 0 {
            return Err(Error::Save(format!("UTF-16 FString 不支持 @ 0x{off:x}")));
        }
        if len == 0 {
            return Ok((String::new(), 4));
        }
        let n = len as usize;
        if off + 4 + n > bytes.len() {
            return Err(Error::Save(format!("FString 内容越界 @ 0x{off:x} len={n}")));
        }
        let text = String::from_utf8_lossy(&bytes[off + 4..off + 4 + n - 1]).into_owned();
        Ok((text, 4 + n))
    }

    // 跳过 header：magic(4) + saveGameVersion(4) + packageUe4(4) + major(2) + minor(2) + patch(2)
    // + build(4) + engineVersion FString + customVersionFormat(4) + count(4) + count×20 + 根 struct 名
    let mut p = 4 + 4 + 4 + 2 + 2 + 2 + 4;
    let (_, c) = read_fstring(sav_bytes, p)?;
    p += c;
    let custom_count = {
        let off = p + 4;
        u32::from_le_bytes(sav_bytes[off..off + 4].try_into().unwrap())
    };
    p += 4 + 4 + custom_count as usize * 20;
    let (root_struct, c) = read_fstring(sav_bytes, p)?;
    p += c;

    let mut out = format!("根 struct: {root_struct}, 属性起始 0x{p:x}\n");

    // 预取 uesave 的值作为对照基准
    let mut expected: BTreeMap<String, String> = BTreeMap::new();
    for name in save.property_names() {
        if let Ok(v) = save.get(&name) {
            expected.insert(name, format!("{v:?}"));
        }
    }

    // 扫描属性
    let mut idx = 0usize;
    let mut matched = 0usize;
    let mut mismatched = 0usize;
    loop {
        let (name, c1) = match read_fstring(sav_bytes, p) {
            Ok(v) => v,
            Err(e) => {
                out.push_str(&format!("[错误] @ 0x{p:x}: {e}\n"));
                break;
            }
        };
        p += c1;
        if name.is_empty() || name == "None" {
            out.push_str(&format!("[结束] None @ 0x{p:x}\n"));
            break;
        }
        let (ty, c2) = match read_fstring(sav_bytes, p) {
            Ok(v) => v,
            Err(e) => {
                out.push_str(&format!("[错误] type @ 0x{p:x} (属性 {name}): {e}\n"));
                break;
            }
        };
        p += c2;
        let size = u32::from_le_bytes(sav_bytes[p..p + 4].try_into().unwrap()) as usize;
        p += 4;
        let _index = u32::from_le_bytes(sav_bytes[p..p + 4].try_into().unwrap());
        p += 4;
        // data（实测规则）：类型特有字段后统一追加 1 字节对齐/保留；
        // Bool 特殊：值 1 字节 + 1 字节对齐（共 2 字节，值区 size=0）
        let data_start = p;
        match ty.as_str() {
            "BoolProperty" => p += 2,
            "ByteProperty" | "EnumProperty" | "ArrayProperty" | "SetProperty" => {
                let (_, c) = read_fstring(sav_bytes, p)?;
                p += c + 1;
            }
            "StructProperty" => {
                let (_, c) = read_fstring(sav_bytes, p)?;
                p += c + 16 + 1;
            }
            "MapProperty" => {
                let (_, c) = read_fstring(sav_bytes, p)?;
                p += c;
                let (_, c) = read_fstring(sav_bytes, p)?;
                p += c + 1;
            }
            _ => p += 1, // Int/Float/Str 等：1 字节
        }
        let data_bytes = &sav_bytes[data_start..p];
        let value_start = p;
        let value_bytes = &sav_bytes[p..p + size];
        if idx < 40 {
            out.push_str(&format!(
                "[{idx:3}] {name} ({ty}) size={size} data@{data_start:x}={data_bytes:02x?} 值区@{value_start:x}={:02x?}\n",
                &value_bytes[..value_bytes.len().min(8)]
            ));
        }
        p += size;

        // 对照 uesave 值（Int/Float 数值对照，其余类型只验证定位）
        if let Some(exp) = expected.get(&name) {
            let ok = match ty.as_str() {
                "IntProperty" if size >= 4 => {
                    let v = i32::from_le_bytes(value_bytes[0..4].try_into().unwrap());
                    format!("Int({v})") == *exp
                }
                "FloatProperty" if size >= 4 => {
                    // f32 → f64 再比较，避免显示精度差异（0.14 vs 0.14000000059604645）
                    let v = f64::from(f32::from_le_bytes(value_bytes[0..4].try_into().unwrap()));
                    exp.trim_start_matches("Float(")
                        .trim_end_matches(')')
                        .parse::<f64>()
                        .map(|e| (v - e).abs() < 1e-6)
                        .unwrap_or(false)
                }
                _ => true,
            };
            if ok {
                matched += 1;
            } else {
                mismatched += 1;
                out.push_str(&format!(
                    "[{idx:3}] {name}: 解析值 ≠ uesave {exp} (type={ty} size={size} data={data_bytes:02x?})\n"
                ));
            }
        }
        idx += 1;
        if idx > 300 {
            return Err(Error::Save("扫描未收敛".into()));
        }
    }
    out.push_str(&format!(
        "扫描完成: {idx} 个属性, 值对照匹配 {matched}, 不匹配 {mismatched}, 结束 @ 0x{p:x} / {}\n",
        sav_bytes.len()
    ));
    Ok(out)
}
