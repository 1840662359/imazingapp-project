//! Save Engine —— UE4 GVAS 存档引擎，uesave-rs 的唯一封装层。
//!
//! 对外提供 [`SaveFile`]，隐藏所有 uesave 实现细节。
//! 其它模块不应直接依赖 uesave，只通过此模块访问 .sav 文件。

use std::io::Cursor;

use domain::error::{Error, Result};
use domain::value::TypedValue;

// uesave 仅在此模块内部可见
use uesave::{Property, PropertyKey, Save};

/// UE4 GVAS 存档的高层封装，提供属性级别的读写接口。
///
/// 内部使用 uesave-rs 解析/序列化，对外隐藏实现细节。
#[derive(Debug)]
pub struct SaveFile {
    /// 内部 uesave Save 结构。
    inner: Save,
}

impl SaveFile {
    /// 从字节流加载 GVAS 存档。
    ///
    /// # Errors
    /// 解析失败时返回 [`domain::Error::Save`]。
    pub fn load(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let save = Save::read(&mut cursor)
            .map_err(|e| Error::Save(format!("uesave parse failed: {}", e)))?;

        Ok(Self { inner: save })
    }

    /// 检查属性是否存在。
    pub fn exists(&self, property_name: &str) -> bool {
        let key = PropertyKey::from(property_name.to_string());
        self.inner.root.properties.0.contains_key(&key)
    }

    /// 获取属性值。
    ///
    /// # Errors
    /// 属性不存在或类型转换失败时返回错误。
    pub fn get(&self, property_name: &str) -> Result<TypedValue> {
        let key = PropertyKey::from(property_name.to_string());
        let prop = self
            .inner
            .root
            .properties
            .0
            .get(&key)
            .ok_or_else(|| Error::Save(format!("property '{}' not found", property_name)))?;

        // Property 是个枚举或包装类型，直接转换
        property_to_typed_value(prop)
    }

    /// 设置属性值。
    ///
    /// 如果属性已存在，更新其值；否则新增属性。
    ///
    /// # Errors
    /// 类型转换失败时返回错误。
    pub fn set(&mut self, property_name: &str, value: TypedValue) -> Result<()> {
        let key = PropertyKey::from(property_name.to_string());
        let uesave_prop = typed_value_to_property(value)?;

        // 直接插入或更新
        self.inner.root.properties.0.insert(key, uesave_prop);

        Ok(())
    }

    /// 删除属性。
    ///
    /// # Errors
    /// 属性不存在时返回错误。
    pub fn remove(&mut self, property_name: &str) -> Result<()> {
        let key = PropertyKey::from(property_name.to_string());

        self.inner
            .root
            .properties
            .0
            .shift_remove(&key)
            .ok_or_else(|| Error::Save(format!("property '{}' not found", property_name)))?;

        Ok(())
    }

    /// 序列化为字节流。
    ///
    /// # Errors
    /// 序列化失败时返回错误。
    pub fn save(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.inner
            .write(&mut buf)
            .map_err(|e| Error::Save(format!("uesave write failed: {}", e)))?;
        Ok(buf)
    }

    /// 获取所有属性名列表。
    pub fn property_names(&self) -> Vec<String> {
        self.inner
            .root
            .properties
            .0
            .keys()
            .map(|k| k.1.clone()) // PropertyKey 是 (u32, String)，第二个是名字
            .collect()
    }

    /// 获取属性数量。
    pub fn len(&self) -> usize {
        self.inner.root.properties.0.len()
    }

    /// 是否为空（无属性）。
    pub fn is_empty(&self) -> bool {
        self.inner.root.properties.0.is_empty()
    }
}

/// 将 uesave 的 Property 转换为 domain::TypedValue。
fn property_to_typed_value(prop: &Property) -> Result<TypedValue> {
    match prop {
        Property::Int(v) => Ok(TypedValue::Int(*v as i64)),
        Property::Int64(v) => Ok(TypedValue::Int(*v)),
        Property::Float(v) => Ok(TypedValue::Float(f64::from(*v))),
        Property::Bool(v) => Ok(TypedValue::Bool(*v)),
        Property::Str(v) => Ok(TypedValue::Str(v.clone())),
        Property::Name(v) => Ok(TypedValue::Str(v.clone())),
        Property::Enum(e) => Ok(TypedValue::Str(e.clone())),
        Property::Byte(_) => {
            // Byte 类型暂不支持，返回占位符
            Ok(TypedValue::Str("<byte>".to_string()))
        }
        _ => Err(Error::Save(format!(
            "unsupported property type: {:?}",
            prop
        ))),
    }
}

/// 将 domain::TypedValue 转换为 uesave 的 Property。
fn typed_value_to_property(value: TypedValue) -> Result<Property> {
    use uesave::Float;

    match value {
        TypedValue::Int(v) => {
            if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
                Ok(Property::Int(v as i32))
            } else {
                Ok(Property::Int64(v))
            }
        }
        TypedValue::Float(v) => Ok(Property::Float(Float(v as f32))),
        TypedValue::Bool(v) => Ok(Property::Bool(v)),
        TypedValue::Str(v) => Ok(Property::Str(v)),
        TypedValue::Enum(_v) => {
            // Enum 类型暂不支持，直接返回错误
            Err(Error::Save(
                "TypedValue::Enum not yet supported for uesave 0.7".to_string(),
            ))
        }
    }
}

/// 从 Property 推断 prop_type 字符串（已废弃，uesave 0.7 不需要）。
#[allow(dead_code)]
fn infer_property_type(value: &Property) -> String {
    match value {
        Property::Int(_) => "IntProperty".to_string(),
        Property::Int64(_) => "Int64Property".to_string(),
        Property::Float(_) => "FloatProperty".to_string(),
        Property::Bool(_) => "BoolProperty".to_string(),
        Property::Str(_) => "StrProperty".to_string(),
        Property::Name(_) => "NameProperty".to_string(),
        Property::Byte { .. } => "ByteProperty".to_string(),
        Property::Enum { .. } => "EnumProperty".to_string(),
        _ => "UnknownProperty".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_round_trip() {
        // 测试类型转换的往返
        let test_cases = vec![
            TypedValue::Int(42),
            TypedValue::Float(3.14),
            TypedValue::Bool(true),
            TypedValue::Str("hello".to_string()),
        ];

        for original in test_cases {
            let prop = typed_value_to_property(original.clone()).unwrap();
            let converted = property_to_typed_value(&prop).unwrap();

            // Float 会有精度损失（f64 → f32 → f64），容差比较
            match (&original, &converted) {
                (TypedValue::Float(a), TypedValue::Float(b)) => {
                    assert!(
                        (a - b).abs() < 0.001,
                        "float mismatch: {} vs {}",
                        a,
                        b
                    );
                }
                _ => assert_eq!(original, converted, "round trip failed"),
            }
        }
    }
}


