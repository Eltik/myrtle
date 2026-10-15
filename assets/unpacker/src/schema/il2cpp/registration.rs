//! `Il2CppMetadataRegistration` in `GameAssembly.dll` and the `Il2CppType`
//! graph it points at.

use anyhow::{Context, Result};

use super::StructuralError;
use super::pe::Pe;

/// `Il2CppTypeEnum` values the generator distinguishes.
pub mod kind {
    pub const VOID: u8 = 0x01;
    pub const BOOLEAN: u8 = 0x02;
    pub const CHAR: u8 = 0x03;
    pub const I1: u8 = 0x04;
    pub const U1: u8 = 0x05;
    pub const I2: u8 = 0x06;
    pub const U2: u8 = 0x07;
    pub const I4: u8 = 0x08;
    pub const U4: u8 = 0x09;
    pub const I8: u8 = 0x0a;
    pub const U8: u8 = 0x0b;
    pub const R4: u8 = 0x0c;
    pub const R8: u8 = 0x0d;
    pub const STRING: u8 = 0x0e;
    pub const PTR: u8 = 0x0f;
    pub const VALUETYPE: u8 = 0x11;
    pub const CLASS: u8 = 0x12;
    pub const VAR: u8 = 0x13;
    pub const ARRAY: u8 = 0x14;
    pub const GENERICINST: u8 = 0x15;
    pub const I: u8 = 0x18;
    pub const U: u8 = 0x19;
    pub const OBJECT: u8 = 0x1c;
    pub const SZARRAY: u8 = 0x1d;
    pub const MVAR: u8 = 0x1e;
    /// Attribute blobs only: a boxed enum value, the enum's type index follows.
    pub const ENUM: u8 = 0x55;
    /// Attribute blobs only: a `System.Type` argument.
    pub const IL2CPP_TYPE_INDEX: u8 = 0xff;
}

/// One `Il2CppType` (16 bytes on x64): an 8-byte data union and a bitfield
/// `attrs:16 type:8 num_mods:5 byref:1 pinned:1 valuetype:1`.
#[derive(Debug, Clone, Copy)]
pub struct RawType {
    pub data: u64,
    /// For a field's type, the `FieldAttributes`.
    pub attrs: u16,
    pub kind: u8,
}

/// A resolved type reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeRef {
    /// A built-in element type (`kind::BOOLEAN` .. `kind::STRING`,
    /// `kind::OBJECT`, `kind::I`, ...).
    Prim(u8),
    /// A class, struct or enum by typedef index.
    Def(u32),
    /// A generic instance: the definition and its arguments.
    Generic(u32, Vec<Self>),
    /// `T[]`.
    SzArray(Box<Self>),
    /// `T[,]` and higher ranks.
    Array(Box<Self>, u8),
    /// A class generic parameter: (`genericParameters` index, position).
    Var(u32, u16),
    /// A method generic parameter or anything else the generator never needs.
    Other(u8),
}

/// Offsets inside `Il2CppMetadataRegistration` (eight count/pointer pairs).
const REG_TYPES: u64 = 6 * 8;
/// `fieldOffsetsCount` sits at qword 10, `typeDefinitionsSizesCount` at 12;
/// both equal the typedef count, which is what the search keys on.
const REG_FIELD_OFFSETS_COUNT: usize = 10;

pub struct Registration {
    pub va: u64,
    /// `types[]`: one pointer per type index.
    pub types: Vec<u64>,
}

fn structural(msg: String) -> anyhow::Error {
    StructuralError(msg).into()
}

/// Find `Il2CppMetadataRegistration` by its `typedefCount, ptr, typedefCount`
/// run (`fieldOffsetsCount`, `fieldOffsets`, `typeDefinitionsSizesCount`)
/// and validate every count/pointer pair before accepting it.
pub fn find(pe: &Pe, typedef_count: usize) -> Result<Registration> {
    let n = typedef_count as u64;
    let mut found: Vec<u64> = Vec::new();
    for s in &pe.sections {
        let start = s.raw_offset as usize;
        let end = (s.raw_offset + s.raw_size).min(pe.data.len() as u64) as usize;
        if start >= end || !start.is_multiple_of(8) {
            continue;
        }
        let words = &pe.data[start..end];
        let q = |i: usize| u64::from_le_bytes(words[i * 8..i * 8 + 8].try_into().unwrap_or([0; 8]));
        let count = words.len() / 8;
        for i in REG_FIELD_OFFSETS_COUNT..count.saturating_sub(3) {
            if q(i) != n || q(i + 2) != n || !pe.is_mapped(q(i + 1)) {
                continue;
            }
            let base = i - REG_FIELD_OFFSETS_COUNT;
            if valid(pe, &(base..base + 16).map(q).collect::<Vec<_>>())
                && let Some(va) = pe.offset_to_va(start + base * 8)
            {
                found.push(va);
            }
        }
    }
    let va = match found.as_slice() {
        [one] => *one,
        [] => {
            return Err(structural(format!(
                "Il2CppMetadataRegistration not found: no `{n}, ptr, {n}` qword run with valid \
                 count/pointer pairs (layout changed or the binary is packed)"
            )));
        }
        many => {
            return Err(structural(format!(
                "Il2CppMetadataRegistration ambiguous: {} candidates at {:x?}",
                many.len(),
                many
            )));
        }
    };
    let types_count = pe.u64_at(va + REG_TYPES)?;
    let types_ptr = pe.u64_at(va + REG_TYPES + 8)?;
    let types = (0..types_count)
        .map(|k| pe.u64_at(types_ptr + k * 8))
        .collect::<Result<Vec<_>>>()
        .context("reading the types[] pointer array")?;
    Ok(Registration { va, types })
}

/// Counts below 10,000,000 with a mapped pointer (or a zero count), for the
/// first seven pairs; the eighth (`metadataUsages`) is empty since v27.
fn valid(pe: &Pe, q: &[u64]) -> bool {
    (0..7).all(|p| {
        let (c, ptr) = (q[p * 2], q[p * 2 + 1]);
        c < 10_000_000 && (c == 0 || pe.is_mapped(ptr))
    }) && q[6] > 0
}

impl Registration {
    pub fn raw(&self, pe: &Pe, index: i32) -> Result<RawType> {
        let ptr = *usize::try_from(index)
            .ok()
            .and_then(|i| self.types.get(i))
            .ok_or_else(|| anyhow::anyhow!("type index {index} out of range"))?;
        raw_at(pe, ptr)
    }
}

pub fn raw_at(pe: &Pe, va: u64) -> Result<RawType> {
    let data = pe.u64_at(va)?;
    let bits = pe.u32_at(va + 8)?;
    Ok(RawType {
        data,
        attrs: (bits & 0xffff) as u16,
        kind: ((bits >> 16) & 0xff) as u8,
    })
}

/// Resolve the `Il2CppType` at `va`.
pub fn resolve_at(pe: &Pe, va: u64) -> Result<TypeRef> {
    resolve(pe, raw_at(pe, va)?)
}

pub fn resolve(pe: &Pe, t: RawType) -> Result<TypeRef> {
    Ok(match t.kind {
        kind::CLASS | kind::VALUETYPE => TypeRef::Def(t.data as u32),
        kind::SZARRAY => TypeRef::SzArray(Box::new(resolve_at(pe, t.data)?)),
        kind::ARRAY => {
            // Il2CppArrayType { Il2CppType* etype; uint8 rank; ... }
            let elem = resolve_at(pe, pe.u64_at(t.data)?)?;
            let rank = (pe.u32_at(t.data + 8)? & 0xff) as u8;
            TypeRef::Array(Box::new(elem), rank)
        }
        kind::GENERICINST => {
            // Il2CppGenericClass (v27+) { Il2CppType* type; context { class_inst;
            // method_inst }; cached_class }; Il2CppGenericInst { argc; argv }.
            let def = match resolve_at(pe, pe.u64_at(t.data)?)? {
                TypeRef::Def(d) => d,
                other => anyhow::bail!("generic class at {:#x} over {other:?}", t.data),
            };
            let inst = pe.u64_at(t.data + 8)?;
            let argc = pe.u64_at(inst)?;
            let argv = pe.u64_at(inst + 8)?;
            if argc > 64 {
                anyhow::bail!("generic instance at {inst:#x} claims {argc} arguments");
            }
            let args = (0..argc)
                .map(|k| resolve_at(pe, pe.u64_at(argv + k * 8)?))
                .collect::<Result<Vec<_>>>()?;
            TypeRef::Generic(def, args)
        }
        kind::VAR => TypeRef::Var(t.data as u32, 0),
        k @ (kind::BOOLEAN..=kind::STRING | kind::OBJECT | kind::I | kind::U | kind::VOID) => {
            TypeRef::Prim(k)
        }
        k => TypeRef::Other(k),
    })
}
