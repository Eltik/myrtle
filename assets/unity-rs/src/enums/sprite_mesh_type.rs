/// Unity SpriteMeshType enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SpriteMeshType {
    /// kSpriteMeshTypeFullRect variant
    kSpriteMeshTypeFullRect = 0,
    /// kSpriteMeshTypeTight variant
    kSpriteMeshTypeTight = 1,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl SpriteMeshType {
    /// Creates a SpriteMeshType from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => SpriteMeshType::kSpriteMeshTypeFullRect,
            1 => SpriteMeshType::kSpriteMeshTypeTight,
            _ => SpriteMeshType::Unknown(value),
        }
    }

    /// Converts the SpriteMeshType to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            SpriteMeshType::kSpriteMeshTypeFullRect => 0,
            SpriteMeshType::kSpriteMeshTypeTight => 1,
            SpriteMeshType::Unknown(value) => *value,
        }
    }
}

impl From<u32> for SpriteMeshType {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<SpriteMeshType> for u32 {
    fn from(value: SpriteMeshType) -> Self {
        value.to_u32()
    }
}
