/// Unity SpritePackingRotation enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SpritePackingRotation {
    /// kSPRNone variant
    kSPRNone = 0,
    /// kSPRFlipHorizontal variant
    kSPRFlipHorizontal = 1,
    /// kSPRFlipVertical variant
    kSPRFlipVertical = 2,
    /// kSPRRotate180 variant
    kSPRRotate180 = 3,
    /// kSPRRotate90 variant
    kSPRRotate90 = 4,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl SpritePackingRotation {
    /// Creates a SpritePackingRotation from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => SpritePackingRotation::kSPRNone,
            1 => SpritePackingRotation::kSPRFlipHorizontal,
            2 => SpritePackingRotation::kSPRFlipVertical,
            3 => SpritePackingRotation::kSPRRotate180,
            4 => SpritePackingRotation::kSPRRotate90,
            _ => SpritePackingRotation::Unknown(value),
        }
    }

    /// Converts the SpritePackingRotation to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            SpritePackingRotation::kSPRNone => 0,
            SpritePackingRotation::kSPRFlipHorizontal => 1,
            SpritePackingRotation::kSPRFlipVertical => 2,
            SpritePackingRotation::kSPRRotate180 => 3,
            SpritePackingRotation::kSPRRotate90 => 4,
            SpritePackingRotation::Unknown(value) => *value,
        }
    }
}

impl From<u32> for SpritePackingRotation {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<SpritePackingRotation> for u32 {
    fn from(value: SpritePackingRotation) -> Self {
        value.to_u32()
    }
}
