// Clippy allows for complex codebase
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::only_used_in_recursion)]

// Module declarations
pub mod classes;
pub mod config;
pub mod enums;
pub mod environment;
pub mod errors;
pub mod export;
pub mod files;
pub mod helpers;
pub mod math;
pub mod streams;
pub mod tools;

// Re-export Environment as the main entry point
pub use environment::Environment;

// Convenience alias (matching Python's `load` function)
pub use environment::Environment as load;

// Backward compatibility alias (matching Python's AssetsManager)
pub use environment::Environment as AssetsManager;

// Re-export helper functions
pub use helpers::archive_storage_manager::set_assetbundle_decrypt_key;

// Re-export error types
pub use errors::{UnityError, UnityResult};

// Re-export config functions
pub use config::{
    get_fallback_version, get_fallback_version_silent, set_fallback_unity_version,
    set_parse_typetree, should_parse_typetree,
};

// Re-export all enums (matching Python's enums/__init__.py)
pub use enums::{
    audio_compression_format::AudioCompressionFormat,
    audio_type::AudioType,
    build_target::BuildTarget,
    class_id_type::ClassIDType,
    compression_flags::{ArchiveFlags, ArchiveFlagsOld, CompressionFlags},
    file_type::FileType,
    gfx_primitive_type::GfxPrimitiveType,
    graphics_format::GraphicsFormat,
    mesh_topology::MeshTopology,
    pass_type::PassType,
    serialized_property_type::SerializedPropertyType,
    shader_compiler_platform::ShaderCompilerPlatform,
    shader_gpu_program_type::ShaderGpuProgramType,
    sprite_mesh_type::SpriteMeshType,
    sprite_packing_mode::SpritePackingMode,
    sprite_packing_rotation::SpritePackingRotation,
    texture_dimension::TextureDimension,
    texture_format::TextureFormat,
    vertex_channel_format::VertexChannelFormat,
    vertex_format::VertexFormat,
    vertex_format2017::VertexFormat2017,
};

// Re-export file types
pub use files::{
    bundle_file::BundleFile,
    file::{DirectoryInfo, File, ParentFile},
    object_reader::ObjectReader,
    serialized_file::SerializedFile,
    web_file::WebFile,
};

// Re-export class types
pub use classes::{object::Object, pptr::PPtr, unknown_object::UnknownObject};

// Re-export ClassIDType mapping functions (add after line 53)
pub use classes::class_id_type_to_class_map::{
    deserialize_typed_object, get_class_for_type, get_class_id_from_name, has_mapping,
};

// Re-export all generated Unity classes module
pub use classes::generated;

// Re-export binary I/O types
pub use streams::{
    endian::Endian,
    endian_reader::{BinaryReader, MemoryReader},
    endian_writer::{BinaryWriter, EndianBinaryWriter},
};

// Re-export math types
pub use math::{
    Color, Color as ColorRGBA, Matrix4x4, Quaternion, Rectangle, Vector2, Vector3, Vector4,
};

// Re-export tools module
pub use tools::extract_assets;
