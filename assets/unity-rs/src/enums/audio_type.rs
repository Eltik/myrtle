/// Unity AudioType enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AudioType {
    /// UNKNOWN variant
    UNKNOWN = 0,
    /// ACC variant
    ACC = 1,
    /// AIFF variant
    AIFF = 2,
    /// IT variant
    IT = 10,
    /// MOD variant
    MOD = 12,
    /// MPEG variant
    MPEG = 13,
    /// OGGVORBIS variant
    OGGVORBIS = 14,
    /// S3M variant
    S3M = 17,
    /// WAV variant
    WAV = 20,
    /// XM variant
    XM = 21,
    /// XMA variant
    XMA = 22,
    /// VAG variant
    VAG = 23,
    /// AUDIOQUEUE variant
    AUDIOQUEUE = 24,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl AudioType {
    /// Creates a AudioType from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => AudioType::UNKNOWN,
            1 => AudioType::ACC,
            2 => AudioType::AIFF,
            10 => AudioType::IT,
            12 => AudioType::MOD,
            13 => AudioType::MPEG,
            14 => AudioType::OGGVORBIS,
            17 => AudioType::S3M,
            20 => AudioType::WAV,
            21 => AudioType::XM,
            22 => AudioType::XMA,
            23 => AudioType::VAG,
            24 => AudioType::AUDIOQUEUE,
            _ => AudioType::Unknown(value),
        }
    }

    /// Converts the AudioType to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            AudioType::UNKNOWN => 0,
            AudioType::ACC => 1,
            AudioType::AIFF => 2,
            AudioType::IT => 10,
            AudioType::MOD => 12,
            AudioType::MPEG => 13,
            AudioType::OGGVORBIS => 14,
            AudioType::S3M => 17,
            AudioType::WAV => 20,
            AudioType::XM => 21,
            AudioType::XMA => 22,
            AudioType::VAG => 23,
            AudioType::AUDIOQUEUE => 24,
            AudioType::Unknown(value) => *value,
        }
    }
}

impl From<u32> for AudioType {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<AudioType> for u32 {
    fn from(value: AudioType) -> Self {
        value.to_u32()
    }
}
