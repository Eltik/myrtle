/// Unity PassType enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PassType {
    /// kPassTypeNormal variant
    kPassTypeNormal = 0,
    /// kPassTypeUse variant
    kPassTypeUse = 1,
    /// kPassTypeGrab variant
    kPassTypeGrab = 2,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl PassType {
    /// Creates a PassType from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => PassType::kPassTypeNormal,
            1 => PassType::kPassTypeUse,
            2 => PassType::kPassTypeGrab,
            _ => PassType::Unknown(value),
        }
    }

    /// Converts the PassType to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            PassType::kPassTypeNormal => 0,
            PassType::kPassTypeUse => 1,
            PassType::kPassTypeGrab => 2,
            PassType::Unknown(value) => *value,
        }
    }
}

impl From<u32> for PassType {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<PassType> for u32 {
    fn from(value: PassType) -> Self {
        value.to_u32()
    }
}
