/// Unity VertexFormat enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum VertexFormat {
    /// kVertexFormatFloat variant
    kVertexFormatFloat = 0,
    /// kVertexFormatFloat16 variant
    kVertexFormatFloat16 = 1,
    /// kVertexFormatUNorm8 variant
    kVertexFormatUNorm8 = 2,
    /// kVertexFormatSNorm8 variant
    kVertexFormatSNorm8 = 3,
    /// kVertexFormatUNorm16 variant
    kVertexFormatUNorm16 = 4,
    /// kVertexFormatSNorm16 variant
    kVertexFormatSNorm16 = 5,
    /// kVertexFormatUInt8 variant
    kVertexFormatUInt8 = 6,
    /// kVertexFormatSInt8 variant
    kVertexFormatSInt8 = 7,
    /// kVertexFormatUInt16 variant
    kVertexFormatUInt16 = 8,
    /// kVertexFormatSInt16 variant
    kVertexFormatSInt16 = 9,
    /// kVertexFormatUInt32 variant
    kVertexFormatUInt32 = 10,
    /// kVertexFormatSInt32 variant
    kVertexFormatSInt32 = 11,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl VertexFormat {
    /// Creates a VertexFormat from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => VertexFormat::kVertexFormatFloat,
            1 => VertexFormat::kVertexFormatFloat16,
            2 => VertexFormat::kVertexFormatUNorm8,
            3 => VertexFormat::kVertexFormatSNorm8,
            4 => VertexFormat::kVertexFormatUNorm16,
            5 => VertexFormat::kVertexFormatSNorm16,
            6 => VertexFormat::kVertexFormatUInt8,
            7 => VertexFormat::kVertexFormatSInt8,
            8 => VertexFormat::kVertexFormatUInt16,
            9 => VertexFormat::kVertexFormatSInt16,
            10 => VertexFormat::kVertexFormatUInt32,
            11 => VertexFormat::kVertexFormatSInt32,
            _ => VertexFormat::Unknown(value),
        }
    }

    /// Converts the VertexFormat to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            VertexFormat::kVertexFormatFloat => 0,
            VertexFormat::kVertexFormatFloat16 => 1,
            VertexFormat::kVertexFormatUNorm8 => 2,
            VertexFormat::kVertexFormatSNorm8 => 3,
            VertexFormat::kVertexFormatUNorm16 => 4,
            VertexFormat::kVertexFormatSNorm16 => 5,
            VertexFormat::kVertexFormatUInt8 => 6,
            VertexFormat::kVertexFormatSInt8 => 7,
            VertexFormat::kVertexFormatUInt16 => 8,
            VertexFormat::kVertexFormatSInt16 => 9,
            VertexFormat::kVertexFormatUInt32 => 10,
            VertexFormat::kVertexFormatSInt32 => 11,
            VertexFormat::Unknown(value) => *value,
        }
    }
}

impl From<u32> for VertexFormat {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<VertexFormat> for u32 {
    fn from(value: VertexFormat) -> Self {
        value.to_u32()
    }
}
