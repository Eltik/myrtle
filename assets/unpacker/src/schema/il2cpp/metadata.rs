//! `global-metadata.dat` v29: header, section table and the rows the schema
//! generator reads. Layout facts are in the parent module's doc.

use std::collections::HashMap;

use anyhow::Result;

use super::StructuralError;

pub const MAGIC: u32 = 0xFAB1_1BAF;
pub const VERSION: u32 = 29;
/// Sections a standard v29 header lists; the CN header lists 33.
pub const STANDARD_SECTIONS: usize = 31;

// Section indices (v29 order).
const S_STRING: usize = 2;
const S_PROPERTIES: usize = 4;
const S_METHODS: usize = 5;
const S_FIELD_DEFAULT_VALUES: usize = 7;
const S_DEFAULT_VALUE_DATA: usize = 8;
const S_FIELDS: usize = 11;
const S_GENERIC_PARAMETERS: usize = 12;
const S_GENERIC_CONTAINERS: usize = 14;
const S_TYPE_DEFINITIONS: usize = 19;
const S_IMAGES: usize = 20;
const S_ATTRIBUTE_DATA: usize = 24;
const S_ATTRIBUTE_DATA_RANGE: usize = 25;

/// The standard v29 `Il2CppTypeDefinition` row; CN adds one int32 (92).
pub const TYPEDEF_STRIDE_STANDARD: usize = 88;
const TYPEDEF_STRIDE_CANDIDATES: [usize; 2] = [88, 92];

#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name_index: u32,
    pub namespace_index: u32,
    pub byval_type_index: i32,
    pub declaring_type_index: i32,
    pub parent_index: i32,
    pub element_type_index: i32,
    pub generic_container_index: i32,
    pub flags: u32,
    pub field_start: i32,
    pub method_start: i32,
    pub property_start: i32,
    pub method_count: u16,
    pub property_count: u16,
    pub field_count: u16,
    pub bitfield: u32,
    pub token: u32,
}

impl TypeDef {
    /// `bitfield` bit 1 is `enumtype`.
    #[must_use]
    pub const fn is_enum(&self) -> bool {
        self.bitfield & 2 != 0
    }
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name_index: u32,
    pub type_index: i32,
    pub token: u32,
}

#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name_index: u32,
    /// Method index relative to the declaring type's `method_start`; -1 none.
    pub get: i32,
    pub set: i32,
    pub token: u32,
}

#[derive(Debug, Clone)]
pub struct MethodDef {
    pub name_index: u32,
    pub declaring_type: i32,
    pub return_type: i32,
    pub flags: u16,
    pub parameter_count: u16,
}

#[derive(Debug, Clone)]
pub struct GenericContainer {
    pub type_argc: i32,
    pub generic_parameter_start: i32,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name_index: u32,
    pub num: u16,
}

#[derive(Debug, Clone)]
pub struct ImageDef {
    pub type_start: u32,
    pub type_count: u32,
    pub custom_attribute_start: u32,
    pub custom_attribute_count: u32,
}

/// A parsed `global-metadata.dat`.
pub struct Metadata {
    data: Vec<u8>,
    pub version: u32,
    pub sections: Vec<Section>,
    pub typedef_stride: usize,
    pub type_defs: Vec<TypeDef>,
    pub fields: Vec<FieldDef>,
    pub properties: Vec<PropertyDef>,
    pub methods: Vec<MethodDef>,
    pub generic_containers: Vec<GenericContainer>,
    pub generic_params: Vec<GenericParam>,
    pub images: Vec<ImageDef>,
    /// Field index -> (type index, offset into the default-value data).
    pub field_defaults: HashMap<u32, (i32, u32)>,
    /// `attributeDataRange` rows: (token, start offset into attribute data).
    pub attribute_ranges: Vec<(u32, u32)>,
}

fn rd_u16(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}
fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}
fn rd_i32(d: &[u8], o: usize) -> i32 {
    rd_u32(d, o) as i32
}

fn structural(msg: String) -> anyhow::Error {
    StructuralError(msg).into()
}

/// Read the header: magic, version and the section table. The section count
/// is implied by where section 0 starts (`(offset - 8) / 8`).
pub fn parse_header(d: &[u8]) -> Result<(u32, Vec<Section>)> {
    if d.len() < 16 {
        return Err(structural(format!(
            "metadata is {} bytes, too short for a header",
            d.len()
        )));
    }
    let magic = rd_u32(d, 0);
    if magic != MAGIC {
        return Err(structural(format!(
            "metadata magic {magic:#010x} != {MAGIC:#010x}: not a plain global-metadata.dat \
             (the Android copy is header-XORed and striped; use the Windows client)"
        )));
    }
    let version = rd_u32(d, 4);
    if version != VERSION {
        return Err(structural(format!(
            "metadata version {version}, this reader knows only v{VERSION}; the client moved to \
             a new IL2CPP version and the row layouts must be re-checked"
        )));
    }
    let first = rd_u32(d, 8) as usize;
    if first < 8 + 8 * STANDARD_SECTIONS || !(first - 8).is_multiple_of(8) || first > d.len() {
        return Err(structural(format!(
            "section 0 starts at {first:#x}; a v29 header needs at least {} sections ({:#x})",
            STANDARD_SECTIONS,
            8 + 8 * STANDARD_SECTIONS
        )));
    }
    let count = (first - 8) / 8;
    let sections: Vec<Section> = (0..count)
        .map(|i| Section {
            offset: rd_u32(d, 8 + 8 * i) as usize,
            size: rd_u32(d, 12 + 8 * i) as usize,
        })
        .collect();
    // Contiguity: every section inside the file, in offset order without
    // overlap, starting right after the header.
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by_key(|&i| (sections[i].offset, sections[i].size));
    let mut end = first;
    for &i in &order {
        let s = sections[i];
        if s.offset < end {
            return Err(structural(format!(
                "section {i} at {:#x} overlaps the previous section ending at {end:#x}",
                s.offset
            )));
        }
        if s.offset - end > 16 {
            return Err(structural(format!(
                "section {i} at {:#x} leaves a {}-byte gap after {end:#x}",
                s.offset,
                s.offset - end
            )));
        }
        end = s.offset + s.size;
        if end > d.len() {
            return Err(structural(format!(
                "section {i} ends at {end:#x}, past the file end {:#x}",
                d.len()
            )));
        }
    }
    Ok((version, sections))
}

fn rows(d: &[u8], s: Section, stride: usize, what: &str) -> Result<usize> {
    if !s.size.is_multiple_of(stride) {
        return Err(structural(format!(
            "{what} section is {} bytes, not a multiple of the {stride}-byte row",
            s.size
        )));
    }
    debug_assert!(s.offset + s.size <= d.len());
    Ok(s.size / stride)
}

fn c_str(d: &[u8], start: usize) -> &[u8] {
    let rest = &d[start.min(d.len())..];
    let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
    &rest[..end]
}

/// Pick the typedef row size: the candidate that divides the section, makes
/// the images' `typeCount` sum equal the row count, and names row 0
/// `<Module>`. Exactly one must pass.
pub fn detect_typedef_stride(d: &[u8], sections: &[Section]) -> Result<usize> {
    let td = sections[S_TYPE_DEFINITIONS];
    let img = sections[S_IMAGES];
    let strings = sections[S_STRING];
    if !img.size.is_multiple_of(40) {
        return Err(structural(format!(
            "images section is {} bytes, not a multiple of 40",
            img.size
        )));
    }
    let image_types: u64 = (0..img.size / 40)
        .map(|k| u64::from(rd_u32(d, img.offset + k * 40 + 12)))
        .sum();
    let mut passing = Vec::new();
    let mut tried = Vec::new();
    for stride in TYPEDEF_STRIDE_CANDIDATES {
        if !td.size.is_multiple_of(stride) || td.size < stride {
            tried.push(format!("{stride}: section size {} not a multiple", td.size));
            continue;
        }
        let n = (td.size / stride) as u64;
        let name_idx = rd_u32(d, td.offset) as usize;
        let name = if name_idx < strings.size {
            c_str(d, strings.offset + name_idx)
        } else {
            &[]
        };
        if n == image_types && name == b"<Module>" {
            passing.push(stride);
        } else {
            tried.push(format!(
                "{stride}: {n} rows vs {image_types} in the images, row 0 named {:?}",
                String::from_utf8_lossy(name)
            ));
        }
    }
    match passing.as_slice() {
        [one] => Ok(*one),
        _ => Err(structural(format!(
            "typedef row size undetermined ({} candidates pass; {})",
            passing.len(),
            tried.join("; ")
        ))),
    }
}

impl Metadata {
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        let (version, sections) = parse_header(&data)?;
        let d = &data;
        let stride = detect_typedef_stride(d, &sections)?;
        let extra = stride - TYPEDEF_STRIDE_STANDARD;

        let td = sections[S_TYPE_DEFINITIONS];
        let n = td.size / stride;
        let type_defs: Vec<TypeDef> = (0..n)
            .map(|k| {
                let o = td.offset + k * stride;
                let c = o + 64 + extra; // the ushort counts follow the extra column
                TypeDef {
                    name_index: rd_u32(d, o),
                    namespace_index: rd_u32(d, o + 4),
                    byval_type_index: rd_i32(d, o + 8),
                    declaring_type_index: rd_i32(d, o + 12),
                    parent_index: rd_i32(d, o + 16),
                    element_type_index: rd_i32(d, o + 20),
                    generic_container_index: rd_i32(d, o + 24),
                    flags: rd_u32(d, o + 28),
                    field_start: rd_i32(d, o + 32),
                    method_start: rd_i32(d, o + 36),
                    property_start: rd_i32(d, o + 44),
                    method_count: rd_u16(d, c),
                    property_count: rd_u16(d, c + 2),
                    field_count: rd_u16(d, c + 4),
                    bitfield: rd_u32(d, c + 16),
                    token: rd_u32(d, c + 20),
                }
            })
            .collect();

        let s = sections[S_FIELDS];
        let fields = (0..rows(d, s, 12, "fields")?)
            .map(|k| {
                let o = s.offset + k * 12;
                FieldDef {
                    name_index: rd_u32(d, o),
                    type_index: rd_i32(d, o + 4),
                    token: rd_u32(d, o + 8),
                }
            })
            .collect::<Vec<_>>();

        let s = sections[S_PROPERTIES];
        let properties = (0..rows(d, s, 20, "properties")?)
            .map(|k| {
                let o = s.offset + k * 20;
                PropertyDef {
                    name_index: rd_u32(d, o),
                    get: rd_i32(d, o + 4),
                    set: rd_i32(d, o + 8),
                    token: rd_u32(d, o + 16),
                }
            })
            .collect::<Vec<_>>();

        let s = sections[S_METHODS];
        let methods = (0..rows(d, s, 32, "methods")?)
            .map(|k| {
                let o = s.offset + k * 32;
                MethodDef {
                    name_index: rd_u32(d, o),
                    declaring_type: rd_i32(d, o + 4),
                    return_type: rd_i32(d, o + 8),
                    flags: rd_u16(d, o + 24),
                    parameter_count: rd_u16(d, o + 30),
                }
            })
            .collect::<Vec<_>>();

        let s = sections[S_GENERIC_CONTAINERS];
        let generic_containers = (0..rows(d, s, 16, "genericContainers")?)
            .map(|k| {
                let o = s.offset + k * 16;
                GenericContainer {
                    type_argc: rd_i32(d, o + 4),
                    generic_parameter_start: rd_i32(d, o + 12),
                }
            })
            .collect::<Vec<_>>();

        let s = sections[S_GENERIC_PARAMETERS];
        let generic_params = (0..rows(d, s, 16, "genericParameters")?)
            .map(|k| {
                let o = s.offset + k * 16;
                GenericParam {
                    name_index: rd_u32(d, o + 4),
                    num: rd_u16(d, o + 12),
                }
            })
            .collect::<Vec<_>>();

        let s = sections[S_IMAGES];
        let images = (0..rows(d, s, 40, "images")?)
            .map(|k| {
                let o = s.offset + k * 40;
                ImageDef {
                    type_start: rd_u32(d, o + 8),
                    type_count: rd_u32(d, o + 12),
                    custom_attribute_start: rd_u32(d, o + 32),
                    custom_attribute_count: rd_u32(d, o + 36),
                }
            })
            .collect::<Vec<_>>();

        let s = sections[S_FIELD_DEFAULT_VALUES];
        let field_defaults = (0..rows(d, s, 12, "fieldDefaultValues")?)
            .map(|k| {
                let o = s.offset + k * 12;
                (rd_u32(d, o), (rd_i32(d, o + 4), rd_u32(d, o + 8)))
            })
            .collect::<HashMap<_, _>>();

        let s = sections[S_ATTRIBUTE_DATA_RANGE];
        let attribute_ranges = (0..rows(d, s, 8, "attributeDataRange")?)
            .map(|k| {
                let o = s.offset + k * 8;
                (rd_u32(d, o), rd_u32(d, o + 4))
            })
            .collect::<Vec<_>>();

        let meta = Self {
            data,
            version,
            sections,
            typedef_stride: stride,
            type_defs,
            fields,
            properties,
            methods,
            generic_containers,
            generic_params,
            images,
            field_defaults,
            attribute_ranges,
        };
        meta.check_ranges()?;
        Ok(meta)
    }

    /// Every typedef's member ranges must sit inside their tables.
    fn check_ranges(&self) -> Result<()> {
        for (k, t) in self.type_defs.iter().enumerate() {
            let fits = |start: i32, count: u16, len: usize| {
                count == 0 || (start >= 0 && start as usize + count as usize <= len)
            };
            if !fits(t.field_start, t.field_count, self.fields.len())
                || !fits(t.method_start, t.method_count, self.methods.len())
                || !fits(t.property_start, t.property_count, self.properties.len())
            {
                return Err(structural(format!(
                    "typedef {k} member ranges fall outside their tables (row layout changed?)"
                )));
            }
        }
        for (k, img) in self.images.iter().enumerate() {
            if (img.custom_attribute_start + img.custom_attribute_count) as usize
                > self.attribute_ranges.len()
            {
                return Err(structural(format!(
                    "image {k} attribute ranges fall outside attributeDataRange"
                )));
            }
        }
        Ok(())
    }

    /// A NUL-terminated string from the string table.
    #[must_use]
    pub fn string(&self, index: u32) -> &str {
        let s = self.sections[S_STRING];
        let i = index as usize;
        if i >= s.size {
            return "";
        }
        std::str::from_utf8(c_str(&self.data[..s.offset + s.size], s.offset + i)).unwrap_or("")
    }

    /// The default-value data blob from `offset` on.
    #[must_use]
    pub fn default_value_data(&self, offset: u32) -> &[u8] {
        let s = self.sections[S_DEFAULT_VALUE_DATA];
        let o = (offset as usize).min(s.size);
        &self.data[s.offset + o..s.offset + s.size]
    }

    /// Attribute blob `[start, end)` of the attribute-data section.
    #[must_use]
    pub fn attribute_data(&self, start: u32, end: Option<u32>) -> &[u8] {
        let s = self.sections[S_ATTRIBUTE_DATA];
        let a = (start as usize).min(s.size);
        let b = end.map_or(s.size, |e| (e as usize).min(s.size)).max(a);
        &self.data[s.offset + a..s.offset + b]
    }
}

/// A little-endian cursor over a metadata blob, with IL2CPP's compressed
/// integer encoding.
pub struct Blob<'a> {
    d: &'a [u8],
    pub pos: usize,
}

impl<'a> Blob<'a> {
    #[must_use]
    pub const fn new(d: &'a [u8]) -> Self {
        Self { d, pos: 0 }
    }

    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.d.len() {
            anyhow::bail!(
                "blob read of {n} bytes at {} past its end {}",
                self.pos,
                self.d.len()
            );
        }
        let s = &self.d[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16> {
        let b = self.bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn u32(&mut self) -> Result<u32> {
        let b = self.bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn u64(&mut self) -> Result<u64> {
        let b = self.bytes(8)?;
        Ok(u64::from_le_bytes(b.try_into()?))
    }

    /// `ReadCompressedUInt32`: 1, 2 or 4 bytes big-endian by prefix bits,
    /// `0xF0` + raw u32, `0xFE`/`0xFF` for `u32::MAX - 1`/`u32::MAX`.
    pub fn compressed_u32(&mut self) -> Result<u32> {
        let b = self.u8()?;
        Ok(if b & 0x80 == 0 {
            u32::from(b)
        } else if b & 0xC0 == 0x80 {
            (u32::from(b & 0x3F) << 8) | u32::from(self.u8()?)
        } else if b & 0xE0 == 0xC0 {
            let r = self.bytes(3)?;
            (u32::from(b & 0x1F) << 24)
                | (u32::from(r[0]) << 16)
                | (u32::from(r[1]) << 8)
                | u32::from(r[2])
        } else if b == 0xF0 {
            self.u32()?
        } else if b == 0xFE {
            u32::MAX - 1
        } else if b == 0xFF {
            u32::MAX
        } else {
            anyhow::bail!("bad compressed integer prefix {b:#04x}")
        })
    }

    /// `ReadCompressedInt32`: zigzag-like, low bit is the sign.
    pub fn compressed_i32(&mut self) -> Result<i32> {
        let e = self.compressed_u32()?;
        if e == u32::MAX {
            return Ok(i32::MIN);
        }
        let v = (e >> 1) as i32;
        Ok(if e & 1 != 0 { -v - 1 } else { v })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Build a metadata file with `sections` header entries, one image of one
    /// type named `<Module>`, and typedef rows of `stride` bytes.
    pub fn synthetic(sections: usize, stride: usize) -> Vec<u8> {
        let header = 8 + 8 * sections;
        let mut body: Vec<Vec<u8>> = vec![Vec::new(); sections];
        body[S_STRING] = b"\0<Module>\0".to_vec();
        let mut td = vec![0u8; stride];
        td[0..4].copy_from_slice(&1u32.to_le_bytes());
        // field/method/property starts = -1 is fine with zero counts
        body[S_TYPE_DEFINITIONS] = td;
        let mut img = vec![0u8; 40];
        img[12..16].copy_from_slice(&1u32.to_le_bytes());
        body[S_IMAGES] = img;
        let mut out = Vec::new();
        out.extend_from_slice(&MAGIC.to_le_bytes());
        out.extend_from_slice(&VERSION.to_le_bytes());
        let mut off = header;
        let mut table = Vec::new();
        for b in &body {
            table.push((off as u32, b.len() as u32));
            off += b.len().div_ceil(4) * 4;
        }
        for (o, s) in &table {
            out.extend_from_slice(&o.to_le_bytes());
            out.extend_from_slice(&s.to_le_bytes());
        }
        for b in &body {
            out.extend_from_slice(b);
            out.resize(out.len().div_ceil(4) * 4, 0);
        }
        out
    }

    #[test]
    fn parses_standard_and_cn_headers() {
        for (sections, stride) in [(31, 88), (33, 92)] {
            let m = Metadata::parse(synthetic(sections, stride)).unwrap();
            assert_eq!(m.version, 29);
            assert_eq!(m.sections.len(), sections);
            assert_eq!(m.typedef_stride, stride);
            assert_eq!(m.type_defs.len(), 1);
            assert_eq!(m.string(m.type_defs[0].name_index), "<Module>");
        }
    }

    #[test]
    fn rejects_bad_magic_version_and_stride() {
        let mut d = synthetic(31, 88);
        d[0] = 0;
        let e = Metadata::parse(d).err().unwrap();
        assert!(e.downcast_ref::<StructuralError>().is_some(), "{e}");

        let mut d = synthetic(31, 88);
        d[4] = 31;
        let e = Metadata::parse(d).err().unwrap();
        assert!(e.to_string().contains("version 31"), "{e}");

        // a 96-byte row is neither 88 nor 92
        let e = Metadata::parse(synthetic(31, 96)).err().unwrap();
        assert!(e.downcast_ref::<StructuralError>().is_some(), "{e}");
        assert!(e.to_string().contains("typedef row size"), "{e}");
    }

    #[test]
    fn rejects_overlapping_sections() {
        let mut d = synthetic(31, 88);
        // point section 3 at section 2's start
        let s2 = d[8 + 16..8 + 20].to_vec();
        d[8 + 24..8 + 28].copy_from_slice(&s2);
        d[8 + 28..8 + 32].copy_from_slice(&4u32.to_le_bytes());
        assert!(Metadata::parse(d).is_err());
    }

    #[test]
    fn compressed_integers() {
        let mut b = Blob::new(&[
            0x05, 0x81, 0x02, 0xC0, 0x01, 0x02, 0x03, 0xF0, 1, 0, 0, 0, 0xFF,
        ]);
        assert_eq!(b.compressed_u32().unwrap(), 5);
        assert_eq!(b.compressed_u32().unwrap(), 0x102);
        assert_eq!(b.compressed_u32().unwrap(), 0x010203);
        assert_eq!(b.compressed_u32().unwrap(), 1);
        assert_eq!(b.compressed_u32().unwrap(), u32::MAX);
        let mut b = Blob::new(&[0x00, 0x01, 0x02, 0x03, 0xFF]);
        assert_eq!(b.compressed_i32().unwrap(), 0);
        assert_eq!(b.compressed_i32().unwrap(), -1);
        assert_eq!(b.compressed_i32().unwrap(), 1);
        assert_eq!(b.compressed_i32().unwrap(), -2);
        assert_eq!(b.compressed_i32().unwrap(), i32::MIN);
    }
}
