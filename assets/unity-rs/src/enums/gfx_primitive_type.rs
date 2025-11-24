/// Unity GfxPrimitiveType enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GfxPrimitiveType {
    /// kPrimitiveTriangles variant
    kPrimitiveTriangles = 0,
    /// kPrimitiveTriangleStrip variant
    kPrimitiveTriangleStrip = 1,
    /// kPrimitiveQuads variant
    kPrimitiveQuads = 2,
    /// kPrimitiveLines variant
    kPrimitiveLines = 3,
    /// kPrimitiveLineStrip variant
    kPrimitiveLineStrip = 4,
    /// kPrimitivePoints variant
    kPrimitivePoints = 5,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl GfxPrimitiveType {
    /// Creates a GfxPrimitiveType from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => GfxPrimitiveType::kPrimitiveTriangles,
            1 => GfxPrimitiveType::kPrimitiveTriangleStrip,
            2 => GfxPrimitiveType::kPrimitiveQuads,
            3 => GfxPrimitiveType::kPrimitiveLines,
            4 => GfxPrimitiveType::kPrimitiveLineStrip,
            5 => GfxPrimitiveType::kPrimitivePoints,
            _ => GfxPrimitiveType::Unknown(value),
        }
    }

    /// Converts the GfxPrimitiveType to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            GfxPrimitiveType::kPrimitiveTriangles => 0,
            GfxPrimitiveType::kPrimitiveTriangleStrip => 1,
            GfxPrimitiveType::kPrimitiveQuads => 2,
            GfxPrimitiveType::kPrimitiveLines => 3,
            GfxPrimitiveType::kPrimitiveLineStrip => 4,
            GfxPrimitiveType::kPrimitivePoints => 5,
            GfxPrimitiveType::Unknown(value) => *value,
        }
    }
}

impl From<u32> for GfxPrimitiveType {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<GfxPrimitiveType> for u32 {
    fn from(value: GfxPrimitiveType) -> Self {
        value.to_u32()
    }
}
