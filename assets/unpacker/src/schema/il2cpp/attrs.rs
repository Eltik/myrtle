//! v29 custom-attribute blobs and constant values.
//!
//! A member's blob (found by image + metadata token in `attributeDataRange`)
//! is: compressed count `n`; `n` uint32 constructor method indices; then per
//! attribute compressed `argc`, `fieldc`, `propc`, the positional values, and
//! the named values (each a value followed by a compressed member index:
//! `>= 0` relative to the attribute type, else `-(i + 1)` plus a compressed
//! typedef index of the declaring class).

use anyhow::{Result, bail};

use super::Il2Cpp;
use super::metadata::Blob;
use super::registration::kind;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Str(String),
    /// A `System.Type` argument, as a type index.
    Type(i32),
    Array(Vec<Self>),
}

impl Value {
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The decimal text Mono.Cecil's `Constant.ToString()` gives an integer.
    #[must_use]
    pub fn to_constant_string(&self) -> String {
        match self {
            Self::Int(v) => v.to_string(),
            Self::UInt(v) => v.to_string(),
            Self::Bool(b) => if *b { "True" } else { "False" }.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Str(s) => s.clone(),
            Self::Null => String::new(),
            Self::Type(t) => format!("type#{t}"),
            Self::Array(a) => format!("array[{}]", a.len()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    /// Typedef index of the attribute class.
    pub type_def: u32,
    pub args: Vec<Value>,
    /// Named field and property arguments.
    pub named: Vec<(String, Value)>,
}

/// Read one constant of element type `k` (v29 encoding: 4-byte integers are
/// compressed, strings are a compressed length then UTF-8).
pub fn read_value(il: &Il2Cpp, b: &mut Blob<'_>, k: u8) -> Result<Value> {
    Ok(match k {
        kind::BOOLEAN => Value::Bool(b.u8()? != 0),
        kind::U1 => Value::UInt(u64::from(b.u8()?)),
        kind::I1 => Value::Int(i64::from(b.u8()? as i8)),
        kind::CHAR | kind::U2 => Value::UInt(u64::from(b.u16()?)),
        kind::I2 => Value::Int(i64::from(b.u16()? as i16)),
        kind::U4 => Value::UInt(u64::from(b.compressed_u32()?)),
        kind::I4 => Value::Int(i64::from(b.compressed_i32()?)),
        kind::U8 => Value::UInt(b.u64()?),
        kind::I8 => Value::Int(b.u64()? as i64),
        kind::R4 => Value::Float(f64::from(f32::from_le_bytes(b.bytes(4)?.try_into()?))),
        kind::R8 => Value::Float(f64::from_le_bytes(b.bytes(8)?.try_into()?)),
        kind::STRING => {
            let len = b.compressed_i32()?;
            if len < 0 {
                Value::Null
            } else {
                Value::Str(String::from_utf8_lossy(b.bytes(len as usize)?).into_owned())
            }
        }
        kind::SZARRAY => {
            let len = b.compressed_i32()?;
            if len < 0 {
                return Ok(Value::Null);
            }
            let elem = read_encoded_kind(il, b)?;
            let mixed = b.u8()? == 1;
            let mut out = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let k = if mixed {
                    read_encoded_kind(il, b)?
                } else {
                    elem
                };
                out.push(read_value(il, b, k)?);
            }
            Value::Array(out)
        }
        kind::IL2CPP_TYPE_INDEX => {
            let t = b.compressed_i32()?;
            if t < 0 { Value::Null } else { Value::Type(t) }
        }
        other => bail!("unsupported constant element type {other:#04x}"),
    })
}

/// An encoded element type: one byte, or `ENUM` + a compressed type index
/// whose enum's underlying type is the real kind.
fn read_encoded_kind(il: &Il2Cpp, b: &mut Blob<'_>) -> Result<u8> {
    let k = b.u8()?;
    if k != kind::ENUM {
        return Ok(k);
    }
    let type_index = b.compressed_i32()?;
    let raw = il.raw_type(type_index)?;
    let def = il
        .meta
        .type_defs
        .get(raw.data as usize)
        .ok_or_else(|| anyhow::anyhow!("enum type index {type_index} has no typedef"))?;
    Ok(il.raw_type(def.element_type_index)?.kind)
}

/// Decode every attribute in one member's blob.
pub fn decode(il: &Il2Cpp, blob: &[u8]) -> Result<Vec<Attribute>> {
    let mut b = Blob::new(blob);
    let count = b.compressed_u32()? as usize;
    let mut ctors = Vec::with_capacity(count);
    for _ in 0..count {
        ctors.push(b.u32()?);
    }
    let mut out = Vec::with_capacity(count);
    for ctor in ctors {
        let method = il
            .meta
            .methods
            .get(ctor as usize)
            .ok_or_else(|| anyhow::anyhow!("attribute ctor {ctor} out of range"))?;
        let type_def = method.declaring_type as u32;
        let argc = b.compressed_u32()?;
        let fieldc = b.compressed_u32()?;
        let propc = b.compressed_u32()?;
        let mut args = Vec::new();
        for _ in 0..argc {
            let k = read_encoded_kind(il, &mut b)?;
            args.push(read_value(il, &mut b, k)?);
        }
        let mut named = Vec::new();
        for is_prop in std::iter::repeat_n(false, fieldc as usize)
            .chain(std::iter::repeat_n(true, propc as usize))
        {
            let k = read_encoded_kind(il, &mut b)?;
            let v = read_value(il, &mut b, k)?;
            let mut member = b.compressed_i32()?;
            let mut owner = type_def;
            if member < 0 {
                member = -(member + 1);
                owner = b.compressed_u32()?;
            }
            let t = il
                .meta
                .type_defs
                .get(owner as usize)
                .ok_or_else(|| anyhow::anyhow!("named argument owner {owner} out of range"))?;
            let name_index = if is_prop {
                il.meta
                    .properties
                    .get((t.property_start + member) as usize)
                    .map(|p| p.name_index)
            } else {
                il.meta
                    .fields
                    .get((t.field_start + member) as usize)
                    .map(|f| f.name_index)
            };
            let name = name_index.map_or("", |i| il.meta.string(i)).to_string();
            named.push((name, v));
        }
        out.push(Attribute {
            type_def,
            args,
            named,
        });
    }
    Ok(out)
}
