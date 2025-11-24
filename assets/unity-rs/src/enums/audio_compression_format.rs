/// Unity AudioCompressionFormat enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AudioCompressionFormat {
    /// PCM variant
    PCM = 0,
    /// Vorbis variant
    Vorbis = 1,
    /// ADPCM variant
    ADPCM = 2,
    /// MP3 variant
    MP3 = 3,
    /// VAG variant
    VAG = 4,
    /// HEVAG variant
    HEVAG = 5,
    /// XMA variant
    XMA = 6,
    /// AAC variant
    AAC = 7,
    /// GCADPCM variant
    GCADPCM = 8,
    /// ATRAC9 variant
    ATRAC9 = 9,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl AudioCompressionFormat {
    /// Creates a AudioCompressionFormat from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => AudioCompressionFormat::PCM,
            1 => AudioCompressionFormat::Vorbis,
            2 => AudioCompressionFormat::ADPCM,
            3 => AudioCompressionFormat::MP3,
            4 => AudioCompressionFormat::VAG,
            5 => AudioCompressionFormat::HEVAG,
            6 => AudioCompressionFormat::XMA,
            7 => AudioCompressionFormat::AAC,
            8 => AudioCompressionFormat::GCADPCM,
            9 => AudioCompressionFormat::ATRAC9,
            _ => AudioCompressionFormat::Unknown(value),
        }
    }

    /// Converts the AudioCompressionFormat to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            AudioCompressionFormat::PCM => 0,
            AudioCompressionFormat::Vorbis => 1,
            AudioCompressionFormat::ADPCM => 2,
            AudioCompressionFormat::MP3 => 3,
            AudioCompressionFormat::VAG => 4,
            AudioCompressionFormat::HEVAG => 5,
            AudioCompressionFormat::XMA => 6,
            AudioCompressionFormat::AAC => 7,
            AudioCompressionFormat::GCADPCM => 8,
            AudioCompressionFormat::ATRAC9 => 9,
            AudioCompressionFormat::Unknown(value) => *value,
        }
    }
}

impl From<u32> for AudioCompressionFormat {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<AudioCompressionFormat> for u32 {
    fn from(value: AudioCompressionFormat) -> Self {
        value.to_u32()
    }
}
