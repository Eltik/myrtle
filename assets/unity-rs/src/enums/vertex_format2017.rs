/// Unity VertexFormat2017 enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum VertexFormat2017 {
    /// kVertexFormatFloat variant
    kVertexFormatFloat = 0,
    /// kVertexFormatFloat16 variant
    kVertexFormatFloat16 = 1,
    /// kVertexFormatColor variant
    kVertexFormatColor = 2,
    /// kVertexFormatUNorm8 variant
    kVertexFormatUNorm8 = 3,
    /// kVertexFormatSNorm8 variant
    kVertexFormatSNorm8 = 4,
    /// kVertexFormatUNorm16 variant
    kVertexFormatUNorm16 = 5,
    /// kVertexFormatSNorm16 variant
    kVertexFormatSNorm16 = 6,
    /// kVertexFormatUInt8 variant
    kVertexFormatUInt8 = 7,
    /// kVertexFormatSInt8 variant
    kVertexFormatSInt8 = 8,
    /// kVertexFormatUInt16 variant
    kVertexFormatUInt16 = 9,
    /// kVertexFormatSInt16 variant
    kVertexFormatSInt16 = 10,
    /// kVertexFormatUInt32 variant
    kVertexFormatUInt32 = 11,
    /// kVertexFormatSInt32 variant
    kVertexFormatSInt32 = 12,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl VertexFormat2017 {
    /// Creates a VertexFormat2017 from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => VertexFormat2017::kVertexFormatFloat,
            1 => VertexFormat2017::kVertexFormatFloat16,
            2 => VertexFormat2017::kVertexFormatColor,
            3 => VertexFormat2017::kVertexFormatUNorm8,
            4 => VertexFormat2017::kVertexFormatSNorm8,
            5 => VertexFormat2017::kVertexFormatUNorm16,
            6 => VertexFormat2017::kVertexFormatSNorm16,
            7 => VertexFormat2017::kVertexFormatUInt8,
            8 => VertexFormat2017::kVertexFormatSInt8,
            9 => VertexFormat2017::kVertexFormatUInt16,
            10 => VertexFormat2017::kVertexFormatSInt16,
            11 => VertexFormat2017::kVertexFormatUInt32,
            12 => VertexFormat2017::kVertexFormatSInt32,
            _ => VertexFormat2017::Unknown(value),
        }
    }

    /// Converts the VertexFormat2017 to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            VertexFormat2017::kVertexFormatFloat => 0,
            VertexFormat2017::kVertexFormatFloat16 => 1,
            VertexFormat2017::kVertexFormatColor => 2,
            VertexFormat2017::kVertexFormatUNorm8 => 3,
            VertexFormat2017::kVertexFormatSNorm8 => 4,
            VertexFormat2017::kVertexFormatUNorm16 => 5,
            VertexFormat2017::kVertexFormatSNorm16 => 6,
            VertexFormat2017::kVertexFormatUInt8 => 7,
            VertexFormat2017::kVertexFormatSInt8 => 8,
            VertexFormat2017::kVertexFormatUInt16 => 9,
            VertexFormat2017::kVertexFormatSInt16 => 10,
            VertexFormat2017::kVertexFormatUInt32 => 11,
            VertexFormat2017::kVertexFormatSInt32 => 12,
            VertexFormat2017::Unknown(value) => *value,
        }
    }
}

impl From<u32> for VertexFormat2017 {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<VertexFormat2017> for u32 {
    fn from(value: VertexFormat2017) -> Self {
        value.to_u32()
    }
}
