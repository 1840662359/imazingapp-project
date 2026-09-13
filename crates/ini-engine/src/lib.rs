//! Ini Engine - UserCustom.ini 文件处理
//!
//! 用于解析和修改和平精英的 UserCustom.ini 配置文件。
//!
//! ## 文件格式
//!
//! ```ini
//! [UserCustom DeviceProfile]
//! +CVars=r.MobileContentScaleFactor=2.0
//! +CVars=sg.ResolutionQuality=100
//! +CVars=t.MaxFPS=90
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use ini_engine::IniDoc;
//! # use domain::error::Result;
//!
//! # fn example() -> Result<()> {
//! # let ini_bytes = b"[UserCustom DeviceProfile]\n+CVars=t.MaxFPS=60\n";
//! // 加载
//! let mut doc = IniDoc::load(ini_bytes)?;
//!
//! // 修改
//! doc.set("UserCustom DeviceProfile", "r.MobileContentScaleFactor", "2.0")?;
//! doc.set("UserCustom DeviceProfile", "sg.ResolutionQuality", "100")?;
//!
//! // 删除
//! doc.remove("UserCustom DeviceProfile", "t.MaxFPS")?;
//!
//! // 保存
//! let new_bytes = doc.save()?;
//! # Ok(())
//! # }
//! ```

use domain::error::{Error, Result};
use std::collections::HashMap;

/// Ini 文档
#[derive(Debug, Clone)]
pub struct IniDoc {
    /// Section 名称 -> (Key -> Value)
    sections: HashMap<String, HashMap<String, String>>,
    /// 保留原始 section 顺序
    section_order: Vec<String>,
}

impl IniDoc {
    /// 从字节流加载 ini 文件
    ///
    /// ## 示例
    ///
    /// ```
    /// # use ini_engine::IniDoc;
    /// # use domain::error::Result;
    /// # fn example() -> Result<()> {
    /// let doc = IniDoc::load(b"[Section]\n+CVars=key=value\n")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load(bytes: &[u8]) -> Result<Self> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| Error::Validation(format!("Invalid UTF-8: {}", e)))?;

        let mut sections = HashMap::new();
        let mut section_order = Vec::new();
        let mut current_section: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            // Section header: [SectionName]
            if line.starts_with('[') && line.ends_with(']') {
                let section_name = line[1..line.len() - 1].to_string();
                current_section = Some(section_name.clone());

                if !sections.contains_key(&section_name) {
                    sections.insert(section_name.clone(), HashMap::new());
                    section_order.push(section_name);
                }
                continue;
            }

            // Entry: +CVars=key=value
            if let Some(entry) = line.strip_prefix("+CVars=") {
                let Some(ref section_name) = current_section else {
                    return Err(Error::Validation(
                        "Entry found before any section".to_string(),
                    ));
                };

                // 解析 key=value
                if let Some((key, value)) = entry.split_once('=') {
                    sections
                        .get_mut(section_name)
                        .unwrap()
                        .insert(key.to_string(), value.to_string());
                } else {
                    return Err(Error::Validation(format!("Invalid entry format: {}", line)));
                }
            }
        }

        Ok(Self {
            sections,
            section_order,
        })
    }

    /// 设置值
    ///
    /// ## 参数
    ///
    /// - `section`: Section 名称（如 "UserCustom DeviceProfile"）
    /// - `key`: 键名（如 "r.MobileContentScaleFactor"）
    /// - `value`: 值（如 "2.0"）
    ///
    /// ## 示例
    ///
    /// ```
    /// # use ini_engine::IniDoc;
    /// # use domain::error::Result;
    /// # fn example() -> Result<()> {
    /// # let mut doc = IniDoc::load(b"[UserCustom DeviceProfile]\n+CVars=key=value\n")?;
    /// doc.set("UserCustom DeviceProfile", "r.MobileContentScaleFactor", "2.0")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set(&mut self, section: &str, key: &str, value: &str) -> Result<()> {
        // 如果 section 不存在，创建它
        if !self.sections.contains_key(section) {
            self.sections.insert(section.to_string(), HashMap::new());
            self.section_order.push(section.to_string());
        }

        // 设置值
        self.sections
            .get_mut(section)
            .unwrap()
            .insert(key.to_string(), value.to_string());

        Ok(())
    }

    /// 删除键
    ///
    /// ## 示例
    ///
    /// ```
    /// # use ini_engine::IniDoc;
    /// # use domain::error::Result;
    /// # fn example() -> Result<()> {
    /// # let mut doc = IniDoc::load(b"[UserCustom DeviceProfile]\n+CVars=t.MaxFPS=90\n")?;
    /// doc.remove("UserCustom DeviceProfile", "t.MaxFPS")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove(&mut self, section: &str, key: &str) -> Result<()> {
        if let Some(entries) = self.sections.get_mut(section) {
            entries.remove(key);
        }
        Ok(())
    }

    /// 保存为字节流
    ///
    /// ## 示例
    ///
    /// ```
    /// # use ini_engine::IniDoc;
    /// # use domain::error::Result;
    /// # fn example() -> Result<()> {
    /// # let doc = IniDoc::load(b"[UserCustom DeviceProfile]\n+CVars=key=value\n")?;
    /// let bytes = doc.save()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save(&self) -> Result<Vec<u8>> {
        let mut output = String::new();

        for section_name in &self.section_order {
            if let Some(entries) = self.sections.get(section_name) {
                // Section header
                output.push_str(&format!("[{}]\n", section_name));

                // Entries (按 key 排序，保证确定性)
                let mut keys: Vec<_> = entries.keys().collect();
                keys.sort();

                for key in keys {
                    let value = &entries[key];
                    output.push_str(&format!("+CVars={}={}\n", key, value));
                }

                output.push('\n'); // Section 之间空一行
            }
        }

        Ok(output.into_bytes())
    }

    /// 获取值
    ///
    /// ## 示例
    ///
    /// ```
    /// # use ini_engine::IniDoc;
    /// # use domain::error::Result;
    /// # fn example() -> Result<()> {
    /// # let doc = IniDoc::load(b"[UserCustom DeviceProfile]\n+CVars=r.MobileContentScaleFactor=2.0\n")?;
    /// let value = doc.get("UserCustom DeviceProfile", "r.MobileContentScaleFactor");
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|entries| entries.get(key))
            .map(|s| s.as_str())
    }

    /// 检查键是否存在
    pub fn contains(&self, section: &str, key: &str) -> bool {
        self.sections
            .get(section)
            .map(|entries| entries.contains_key(key))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty() {
        let doc = IniDoc::load(b"").unwrap();
        assert_eq!(doc.sections.len(), 0);
    }

    #[test]
    fn test_load_single_section() {
        let ini = b"[UserCustom DeviceProfile]\n+CVars=r.MobileContentScaleFactor=2.0\n";
        let doc = IniDoc::load(ini).unwrap();

        assert_eq!(doc.sections.len(), 1);
        assert_eq!(
            doc.get("UserCustom DeviceProfile", "r.MobileContentScaleFactor"),
            Some("2.0")
        );
    }

    #[test]
    fn test_load_multiple_entries() {
        let ini = b"[UserCustom DeviceProfile]\n\
                    +CVars=r.MobileContentScaleFactor=2.0\n\
                    +CVars=sg.ResolutionQuality=100\n\
                    +CVars=t.MaxFPS=90\n";

        let doc = IniDoc::load(ini).unwrap();

        assert_eq!(
            doc.get("UserCustom DeviceProfile", "r.MobileContentScaleFactor"),
            Some("2.0")
        );
        assert_eq!(
            doc.get("UserCustom DeviceProfile", "sg.ResolutionQuality"),
            Some("100")
        );
        assert_eq!(doc.get("UserCustom DeviceProfile", "t.MaxFPS"), Some("90"));
    }

    #[test]
    fn test_set() {
        let mut doc = IniDoc::load(b"").unwrap();

        doc.set(
            "UserCustom DeviceProfile",
            "r.MobileContentScaleFactor",
            "2.0",
        )
        .unwrap();

        assert_eq!(
            doc.get("UserCustom DeviceProfile", "r.MobileContentScaleFactor"),
            Some("2.0")
        );
    }

    #[test]
    fn test_set_overwrite() {
        let ini = b"[UserCustom DeviceProfile]\n+CVars=r.MobileContentScaleFactor=1.0\n";
        let mut doc = IniDoc::load(ini).unwrap();

        doc.set(
            "UserCustom DeviceProfile",
            "r.MobileContentScaleFactor",
            "2.0",
        )
        .unwrap();

        assert_eq!(
            doc.get("UserCustom DeviceProfile", "r.MobileContentScaleFactor"),
            Some("2.0")
        );
    }

    #[test]
    fn test_set_inserts_missing_key_without_losing_existing_entries() {
        let ini = b"[UserCustom DeviceProfile]\n+CVars=r.UserQualitySetting=1\n";
        let mut doc = IniDoc::load(ini).unwrap();

        doc.set("UserCustom DeviceProfile", "r.PUBGDeviceFPSDef", "8")
            .unwrap();

        assert_eq!(
            doc.get("UserCustom DeviceProfile", "r.UserQualitySetting"),
            Some("1")
        );
        assert_eq!(
            doc.get("UserCustom DeviceProfile", "r.PUBGDeviceFPSDef"),
            Some("8")
        );
    }

    #[test]
    fn test_set_creates_missing_section_and_serializes_it() {
        let mut doc = IniDoc::load(b"[Other]\n+CVars=keep=value\n").unwrap();

        doc.set("UserCustom DeviceProfile", "r.PUBGDeviceFPSDef", "8")
            .unwrap();
        let content = String::from_utf8(doc.save().unwrap()).unwrap();

        assert!(content.contains("[Other]\n+CVars=keep=value"));
        assert!(content.contains(
            "[UserCustom DeviceProfile]\n+CVars=r.PUBGDeviceFPSDef=8"
        ));
    }

    #[test]
    fn test_remove() {
        let ini = b"[UserCustom DeviceProfile]\n\
                    +CVars=r.MobileContentScaleFactor=2.0\n\
                    +CVars=t.MaxFPS=90\n";

        let mut doc = IniDoc::load(ini).unwrap();

        doc.remove("UserCustom DeviceProfile", "t.MaxFPS").unwrap();

        assert_eq!(doc.get("UserCustom DeviceProfile", "t.MaxFPS"), None);
        assert_eq!(
            doc.get("UserCustom DeviceProfile", "r.MobileContentScaleFactor"),
            Some("2.0")
        );
    }

    #[test]
    fn test_save() {
        let mut doc = IniDoc::load(b"").unwrap();

        doc.set(
            "UserCustom DeviceProfile",
            "r.MobileContentScaleFactor",
            "2.0",
        )
        .unwrap();
        doc.set("UserCustom DeviceProfile", "sg.ResolutionQuality", "100")
            .unwrap();

        let bytes = doc.save().unwrap();
        let content = String::from_utf8(bytes).unwrap();

        assert!(content.contains("[UserCustom DeviceProfile]"));
        assert!(content.contains("+CVars=r.MobileContentScaleFactor=2.0"));
        assert!(content.contains("+CVars=sg.ResolutionQuality=100"));
    }

    #[test]
    fn test_roundtrip() {
        let original = b"[UserCustom DeviceProfile]\n\
                         +CVars=r.MobileContentScaleFactor=2.0\n\
                         +CVars=sg.ResolutionQuality=100\n";

        let doc = IniDoc::load(original).unwrap();
        let bytes = doc.save().unwrap();
        let doc2 = IniDoc::load(&bytes).unwrap();

        assert_eq!(
            doc2.get("UserCustom DeviceProfile", "r.MobileContentScaleFactor"),
            Some("2.0")
        );
        assert_eq!(
            doc2.get("UserCustom DeviceProfile", "sg.ResolutionQuality"),
            Some("100")
        );
    }

    #[test]
    fn test_contains() {
        let ini = b"[UserCustom DeviceProfile]\n+CVars=r.MobileContentScaleFactor=2.0\n";
        let doc = IniDoc::load(ini).unwrap();

        assert!(doc.contains("UserCustom DeviceProfile", "r.MobileContentScaleFactor"));
        assert!(!doc.contains("UserCustom DeviceProfile", "nonexistent"));
    }
}
