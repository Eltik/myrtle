/// Unity TextureDimension enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TextureDimension {
    /// kTexDimUnknown variant
    kTexDimUnknown = -1,
    /// kTexDimNone variant
    kTexDimNone = 0,
    /// kTexDimAny variant
    kTexDimAny = 1,
    /// kTexDim2D variant
    kTexDim2D = 2,
    /// kTexDim3D variant
    kTexDim3D = 3,
    /// kTexDimCUBE variant
    kTexDimCUBE = 4,
    /// kTexDim2DArray variant
    kTexDim2DArray = 5,
    /// kTexDimCubeArray variant
    kTexDimCubeArray = 6,
    /// kTexDimForce32Bit variant
    kTexDimForce32Bit = 2147483647,
}

impl TextureDimension {
    /// Creates a TextureDimension from a i32 value, returns None for unknown values
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            -1 => Some(TextureDimension::kTexDimUnknown),
            0 => Some(TextureDimension::kTexDimNone),
            1 => Some(TextureDimension::kTexDimAny),
            2 => Some(TextureDimension::kTexDim2D),
            3 => Some(TextureDimension::kTexDim3D),
            4 => Some(TextureDimension::kTexDimCUBE),
            5 => Some(TextureDimension::kTexDim2DArray),
            6 => Some(TextureDimension::kTexDimCubeArray),
            2147483647 => Some(TextureDimension::kTexDimForce32Bit),
            _ => None,
        }
    }

    /// Converts the TextureDimension to its i32 representation
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for TextureDimension {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(())
    }
}

impl From<TextureDimension> for i32 {
    fn from(value: TextureDimension) -> Self {
        value.to_i32()
    }
}
