/// Unity VertexChannelFormat enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum VertexChannelFormat {
    /// kChannelFormatFloat variant
    kChannelFormatFloat = 0,
    /// kChannelFormatFloat16 variant
    kChannelFormatFloat16 = 1,
    /// kChannelFormatColor variant
    kChannelFormatColor = 2,
    /// kChannelFormatByte variant
    kChannelFormatByte = 3,
    /// kChannelFormatUInt32 variant
    kChannelFormatUInt32 = 4,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl VertexChannelFormat {
    /// Creates a VertexChannelFormat from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => VertexChannelFormat::kChannelFormatFloat,
            1 => VertexChannelFormat::kChannelFormatFloat16,
            2 => VertexChannelFormat::kChannelFormatColor,
            3 => VertexChannelFormat::kChannelFormatByte,
            4 => VertexChannelFormat::kChannelFormatUInt32,
            _ => VertexChannelFormat::Unknown(value),
        }
    }

    /// Converts the VertexChannelFormat to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            VertexChannelFormat::kChannelFormatFloat => 0,
            VertexChannelFormat::kChannelFormatFloat16 => 1,
            VertexChannelFormat::kChannelFormatColor => 2,
            VertexChannelFormat::kChannelFormatByte => 3,
            VertexChannelFormat::kChannelFormatUInt32 => 4,
            VertexChannelFormat::Unknown(value) => *value,
        }
    }
}

impl From<u32> for VertexChannelFormat {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<VertexChannelFormat> for u32 {
    fn from(value: VertexChannelFormat) -> Self {
        value.to_u32()
    }
}
