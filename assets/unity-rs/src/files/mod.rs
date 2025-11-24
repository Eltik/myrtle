pub mod bundle_file;
pub mod file;
/// Unity asset file structures module
///
/// This module contains types for reading and writing Unity asset files,
/// including SerializedFile (the main asset file format) and ObjectReader
/// (handles individual Unity object deserialization).
pub mod object_reader;
pub mod serialized_file;
pub mod web_file;

pub use file::{DirectoryInfo, File};
pub use object_reader::ObjectReader;
pub use serialized_file::SerializedFile;
pub use serialized_file::{
    BuildType, FileIdentifier, LocalSerializedObjectIdentifier, SerializedFileHeader,
    SerializedType,
};
