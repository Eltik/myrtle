/// Unity SerializedPropertyType enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SerializedPropertyType {
    /// kColor variant
    kColor = 0,
    /// kVector variant
    kVector = 1,
    /// kFloat variant
    kFloat = 2,
    /// kRange variant
    kRange = 3,
    /// kTexture variant
    kTexture = 4,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl SerializedPropertyType {
    /// Creates a SerializedPropertyType from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => SerializedPropertyType::kColor,
            1 => SerializedPropertyType::kVector,
            2 => SerializedPropertyType::kFloat,
            3 => SerializedPropertyType::kRange,
            4 => SerializedPropertyType::kTexture,
            _ => SerializedPropertyType::Unknown(value),
        }
    }

    /// Converts the SerializedPropertyType to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            SerializedPropertyType::kColor => 0,
            SerializedPropertyType::kVector => 1,
            SerializedPropertyType::kFloat => 2,
            SerializedPropertyType::kRange => 3,
            SerializedPropertyType::kTexture => 4,
            SerializedPropertyType::Unknown(value) => *value,
        }
    }
}

impl From<u32> for SerializedPropertyType {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<SerializedPropertyType> for u32 {
    fn from(value: SerializedPropertyType) -> Self {
        value.to_u32()
    }
}
