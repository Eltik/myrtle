/// Unity FileType enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FileType {
    /// AssetsFile variant
    AssetsFile = 0,
    /// BundleFile variant
    BundleFile = 1,
    /// WebFile variant
    WebFile = 2,
    /// ResourceFile variant
    ResourceFile = 9,
    /// ZIP variant
    ZIP = 10,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl FileType {
    /// Creates a FileType from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => FileType::AssetsFile,
            1 => FileType::BundleFile,
            2 => FileType::WebFile,
            9 => FileType::ResourceFile,
            10 => FileType::ZIP,
            _ => FileType::Unknown(value),
        }
    }

    /// Converts the FileType to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            FileType::AssetsFile => 0,
            FileType::BundleFile => 1,
            FileType::WebFile => 2,
            FileType::ResourceFile => 9,
            FileType::ZIP => 10,
            FileType::Unknown(value) => *value,
        }
    }
}

impl From<u32> for FileType {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<FileType> for u32 {
    fn from(value: FileType) -> Self {
        value.to_u32()
    }
}
