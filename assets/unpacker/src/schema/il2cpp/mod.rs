//! A minimal IL2CPP reader over the official CN Windows client
//! (`GameAssembly.dll` + `global-metadata.dat`), scoped to what
//! `schema::gen` needs: type definitions, fields, properties, enum constants,
//! generic instances and custom attributes. No method bodies, no `DummyDll`.
//!
//! Format facts, measured on CN client 2.7.81 (2026-10-15), so the next
//! person does not rediscover them:
//!
//! - Metadata magic `0xFAB11BAF`, version 29. The header is the magic, the
//!   version and (offset, size) int32 pairs; the section count follows from
//!   where section 0 starts. Standard v29 has 31 sections (header 0x100); CN
//!   has 33 (header 0x110), section 31 being 36,944 bytes and section 32
//!   empty. Neither extra section is read. Sections are contiguous in file
//!   order; the parser checks that.
//! - CN typedef rows are 92 bytes, not v29's 88: one extra int32 at dword 16,
//!   after `interfaceOffsetsStart` and before the ushort counts. The stride is
//!   detected, never assumed: of 88 and 92, the one that divides the section,
//!   whose row count equals the images' summed `typeCount` (50,944 on
//!   2.7.81), and whose row 0 is named `<Module>`. Every other table is
//!   standard v29: fields 12 bytes, properties 20, methods 32, generic
//!   containers 16, generic parameters 16, images 40, field default values
//!   12, attribute data ranges 8.
//! - v29 constants (enum values, attribute arguments) encode 4-byte integers
//!   in IL2CPP's compressed form and strings as a compressed length plus
//!   UTF-8.
//! - Field accessibility and `static`/`literal`/`NotSerialized` live in the
//!   `attrs` bits of the field's `Il2CppType` in the binary, not in the
//!   metadata row. Property accessibility is the getter/setter method flags.
//! - `Il2CppMetadataRegistration` is found by scanning the PE for the qword
//!   run `typedefCount, ptr, typedefCount` (`fieldOffsetsCount`,
//!   `fieldOffsets`, `typeDefinitionsSizesCount`); the struct starts 10
//!   qwords earlier, and the first seven count/pointer pairs must be sane. On
//!   2.7.81 it is at VA 0x187c26040. The protector sections (`.tvm0`,
//!   `.Sgxm0`, `.Sgxm1`, `.rsrc2`, `.edata2`) do not disturb it.
//!   `CodeRegistration` is not needed.
//! - v27+ `Il2CppGenericClass` holds an `Il2CppType*` for the definition
//!   (not a typedef index), then `class_inst`.
//! - Attribute types the generator reads: `Newtonsoft.Json.JsonPropertyAttribute`
//!   (ctor string or named `PropertyName`), `Newtonsoft.Json.JsonIgnoreAttribute`,
//!   `UnityEngine.SerializeField` (no `Attribute` suffix in this client),
//!   `UnityEngine.CreateAssetMenuAttribute` (`menuName`, discovery only).
//!
//! Any of these facts failing is a [`StructuralError`]: the client changed
//! shape and a human must look.

pub mod attrs;
pub mod metadata;
pub mod pe;
pub mod registration;

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use anyhow::{Context, Result};

use attrs::{Attribute, Value};
use metadata::{Blob, Metadata};
use pe::Pe;
use registration::{RawType, Registration, TypeRef};

/// The client's binary layout is not what this reader knows. `schema-gen`
/// exits 3 on it.
#[derive(Debug)]
pub struct StructuralError(pub String);

impl fmt::Display for StructuralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "structural detection failed: {}", self.0)
    }
}

impl std::error::Error for StructuralError {}

pub struct Il2Cpp {
    pub meta: Metadata,
    pub pe: Pe,
    pub reg: Registration,
    typedef_image: Vec<u32>,
    attr_index: HashMap<(u32, u32), usize>,
    full_names: Vec<String>,
}

impl Il2Cpp {
    pub fn load(dll: &Path, metadata: &Path) -> Result<Self> {
        let meta =
            std::fs::read(metadata).with_context(|| format!("reading {}", metadata.display()))?;
        let dll = std::fs::read(dll).with_context(|| format!("reading {}", dll.display()))?;
        Self::from_bytes(dll, meta)
    }

    pub fn from_bytes(dll: Vec<u8>, meta: Vec<u8>) -> Result<Self> {
        let meta = Metadata::parse(meta)?;
        let pe = Pe::parse(dll)?;
        let reg = registration::find(&pe, meta.type_defs.len())?;

        let mut typedef_image = vec![u32::MAX; meta.type_defs.len()];
        for (k, img) in meta.images.iter().enumerate() {
            let range = img.type_start as usize..(img.type_start + img.type_count) as usize;
            for t in range {
                if let Some(slot) = typedef_image.get_mut(t) {
                    *slot = k as u32;
                }
            }
        }
        let mut attr_index = HashMap::new();
        for (k, img) in meta.images.iter().enumerate() {
            let start = img.custom_attribute_start as usize;
            for r in start..start + img.custom_attribute_count as usize {
                attr_index.insert((k as u32, meta.attribute_ranges[r].0), r);
            }
        }
        let mut il = Self {
            meta,
            pe,
            reg,
            typedef_image,
            attr_index,
            full_names: Vec::new(),
        };
        il.full_names = (0..il.meta.type_defs.len())
            .map(|k| il.compute_full_name(k as u32))
            .collect();
        il.check_byval_types()?;
        Ok(il)
    }

    /// Every typedef's `byvalTypeIndex` must point back at it through the
    /// binary's `types[]`; a mismatch means the registration or the typedef
    /// layout was misread.
    fn check_byval_types(&self) -> Result<()> {
        let bad = self
            .meta
            .type_defs
            .iter()
            .enumerate()
            .filter(|(k, t)| {
                // Built-ins (`System.Int32`, `System.String`, ...) carry their
                // element type as the kind (18 on 2.7.81); the data is still
                // the typedef index.
                !matches!(self.raw_type(t.byval_type_index), Ok(r) if r.data == *k as u64)
            })
            .count();
        if bad > 0 {
            return Err(StructuralError(format!(
                "{bad} of {} typedefs do not round-trip through types[] (registration at {:#x})",
                self.meta.type_defs.len(),
                self.reg.va
            ))
            .into());
        }
        Ok(())
    }

    fn compute_full_name(&self, def: u32) -> String {
        let t = &self.meta.type_defs[def as usize];
        let name = self.meta.string(t.name_index);
        if t.declaring_type_index >= 0 {
            // Cecil spells nesting `Outer/Inner`; the declaring type is a
            // type index, resolve it to its typedef.
            if let Ok(r) = self.raw_type(t.declaring_type_index)
                && (r.data as usize) < self.meta.type_defs.len()
                && r.data as u32 != def
            {
                return format!("{}/{name}", self.compute_full_name(r.data as u32));
            }
        }
        let ns = self.meta.string(t.namespace_index);
        if ns.is_empty() {
            name.to_string()
        } else {
            format!("{ns}.{name}")
        }
    }

    /// Cecil-style full name, e.g. `Torappu.CharSkinData/DisplaySkin`.
    #[must_use]
    pub fn full_name(&self, def: u32) -> &str {
        self.full_names.get(def as usize).map_or("", String::as_str)
    }

    #[must_use]
    pub fn typedef_count(&self) -> usize {
        self.meta.type_defs.len()
    }

    pub fn raw_type(&self, index: i32) -> Result<RawType> {
        self.reg.raw(&self.pe, index)
    }

    /// Resolve a type index, filling each class generic parameter's
    /// position.
    pub fn type_ref(&self, index: i32) -> Result<TypeRef> {
        let r = registration::resolve(&self.pe, self.raw_type(index)?)?;
        Ok(self.fill_vars(r))
    }

    fn fill_vars(&self, t: TypeRef) -> TypeRef {
        match t {
            TypeRef::Var(p, _) => {
                let num = self
                    .meta
                    .generic_params
                    .get(p as usize)
                    .map_or(0, |g| g.num);
                TypeRef::Var(p, num)
            }
            TypeRef::Generic(d, args) => {
                TypeRef::Generic(d, args.into_iter().map(|a| self.fill_vars(a)).collect())
            }
            TypeRef::SzArray(e) => TypeRef::SzArray(Box::new(self.fill_vars(*e))),
            TypeRef::Array(e, r) => TypeRef::Array(Box::new(self.fill_vars(*e)), r),
            other => other,
        }
    }

    /// Name of a generic parameter by its `genericParameters` index.
    #[must_use]
    pub fn generic_param_name(&self, index: u32) -> &str {
        self.meta
            .generic_params
            .get(index as usize)
            .map_or("", |g| self.meta.string(g.name_index))
    }

    /// The declared generic parameter names of a typedef, in order.
    #[must_use]
    pub fn generic_params_of(&self, def: u32) -> Vec<&str> {
        let t = &self.meta.type_defs[def as usize];
        if t.generic_container_index < 0 {
            return Vec::new();
        }
        let Some(c) = self
            .meta
            .generic_containers
            .get(t.generic_container_index as usize)
        else {
            return Vec::new();
        };
        (0..c.type_argc.max(0))
            .map(|k| self.generic_param_name((c.generic_parameter_start + k) as u32))
            .collect()
    }

    /// The image (assembly module) a typedef belongs to.
    #[must_use]
    pub fn image_of(&self, def: u32) -> u32 {
        self.typedef_image
            .get(def as usize)
            .copied()
            .unwrap_or(u32::MAX)
    }

    /// All custom attributes on the member with `token` in `def`'s image.
    pub fn attributes(&self, def: u32, token: u32) -> Result<Vec<Attribute>> {
        let image = self.image_of(def);
        let Some(&r) = self.attr_index.get(&(image, token)) else {
            return Ok(Vec::new());
        };
        let start = self.meta.attribute_ranges[r].1;
        let end = self.meta.attribute_ranges.get(r + 1).map(|x| x.1);
        attrs::decode(self, self.meta.attribute_data(start, end)).with_context(|| {
            format!(
                "decoding attributes of token {token:#x} in {}",
                self.full_name(def)
            )
        })
    }

    /// A field's constant (`fieldDefaultValues`), for enum members.
    pub fn field_default(&self, field_index: u32) -> Result<Option<Value>> {
        let Some(&(type_index, data)) = self.meta.field_defaults.get(&field_index) else {
            return Ok(None);
        };
        let k = self.raw_type(type_index)?.kind;
        let mut b = Blob::new(self.meta.default_value_data(data));
        attrs::read_value(self, &mut b, k).map(Some)
    }

    /// Every typedef index whose full name is `name`.
    pub fn find_by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = u32> + 'a {
        self.full_names
            .iter()
            .enumerate()
            .filter(move |(_, n)| *n == name)
            .map(|(k, _)| k as u32)
    }
}
