//! Just enough PE32+ to map virtual addresses of `GameAssembly.dll` to file
//! offsets: image base and the section table.

use anyhow::Result;

use super::StructuralError;

#[derive(Debug, Clone)]
pub struct PeSection {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub raw_offset: u64,
    pub raw_size: u64,
}

pub struct Pe {
    pub data: Vec<u8>,
    pub image_base: u64,
    pub sections: Vec<PeSection>,
}

fn rd_u16(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}
fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn structural(msg: String) -> anyhow::Error {
    StructuralError(msg).into()
}

impl Pe {
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        if data.len() < 0x40 || &data[..2] != b"MZ" {
            return Err(structural("GameAssembly.dll has no MZ header".into()));
        }
        let pe = rd_u32(&data, 0x3c) as usize;
        if pe + 24 > data.len() || &data[pe..pe + 4] != b"PE\0\0" {
            return Err(structural("GameAssembly.dll has no PE signature".into()));
        }
        let coff = pe + 4;
        let machine = rd_u16(&data, coff);
        if machine != 0x8664 {
            return Err(structural(format!(
                "GameAssembly.dll machine {machine:#06x}, expected x64 (0x8664)"
            )));
        }
        let nsec = rd_u16(&data, coff + 2) as usize;
        let opt_size = rd_u16(&data, coff + 16) as usize;
        let opt = coff + 20;
        if opt + opt_size > data.len() || rd_u16(&data, opt) != 0x20b {
            return Err(structural("GameAssembly.dll is not PE32+".into()));
        }
        let image_base = u64::from_le_bytes(data[opt + 24..opt + 32].try_into()?);
        let table = opt + opt_size;
        if table + nsec * 40 > data.len() {
            return Err(structural("PE section table runs past the file".into()));
        }
        let sections = (0..nsec)
            .map(|k| {
                let o = table + k * 40;
                let raw_name = &data[o..o + 8];
                let end = raw_name.iter().position(|&b| b == 0).unwrap_or(8);
                PeSection {
                    name: String::from_utf8_lossy(&raw_name[..end]).into_owned(),
                    virtual_size: u64::from(rd_u32(&data, o + 8)),
                    virtual_address: u64::from(rd_u32(&data, o + 12)),
                    raw_size: u64::from(rd_u32(&data, o + 16)),
                    raw_offset: u64::from(rd_u32(&data, o + 20)),
                }
            })
            .collect();
        Ok(Self {
            data,
            image_base,
            sections,
        })
    }

    /// File offset of a virtual address, when it falls in a section's raw
    /// data.
    #[must_use]
    pub fn va_to_offset(&self, va: u64) -> Option<usize> {
        let rva = va.checked_sub(self.image_base)?;
        self.sections.iter().find_map(|s| {
            let delta = rva.checked_sub(s.virtual_address)?;
            (delta < s.raw_size && delta < s.virtual_size.max(s.raw_size))
                .then(|| (s.raw_offset + delta) as usize)
                .filter(|&o| o < self.data.len())
        })
    }

    /// Virtual address of a file offset inside a section's raw data.
    #[must_use]
    pub fn offset_to_va(&self, off: usize) -> Option<u64> {
        let off = off as u64;
        self.sections.iter().find_map(|s| {
            let delta = off.checked_sub(s.raw_offset)?;
            (delta < s.raw_size).then_some(self.image_base + s.virtual_address + delta)
        })
    }

    pub fn u64_at(&self, va: u64) -> Result<u64> {
        let o = self
            .va_to_offset(va)
            .ok_or_else(|| anyhow::anyhow!("address {va:#x} is outside the PE image"))?;
        let b = self
            .data
            .get(o..o + 8)
            .ok_or_else(|| anyhow::anyhow!("address {va:#x} reads past the file"))?;
        Ok(u64::from_le_bytes(b.try_into()?))
    }

    pub fn u32_at(&self, va: u64) -> Result<u32> {
        let o = self
            .va_to_offset(va)
            .ok_or_else(|| anyhow::anyhow!("address {va:#x} is outside the PE image"))?;
        let b = self
            .data
            .get(o..o + 4)
            .ok_or_else(|| anyhow::anyhow!("address {va:#x} reads past the file"))?;
        Ok(u32::from_le_bytes(b.try_into()?))
    }

    #[must_use]
    pub fn is_mapped(&self, va: u64) -> bool {
        self.va_to_offset(va).is_some()
    }
}
