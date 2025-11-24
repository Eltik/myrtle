/// Unity SpritePackingMode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SpritePackingMode {
    /// kSPMTight variant
    kSPMTight = 0,
    /// kSPMRectangle variant
    kSPMRectangle = 1,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl SpritePackingMode {
    /// Creates a SpritePackingMode from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => SpritePackingMode::kSPMTight,
            1 => SpritePackingMode::kSPMRectangle,
            _ => SpritePackingMode::Unknown(value),
        }
    }

    /// Converts the SpritePackingMode to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            SpritePackingMode::kSPMTight => 0,
            SpritePackingMode::kSPMRectangle => 1,
            SpritePackingMode::Unknown(value) => *value,
        }
    }
}

impl From<u32> for SpritePackingMode {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<SpritePackingMode> for u32 {
    fn from(value: SpritePackingMode) -> Self {
        value.to_u32()
    }
}
