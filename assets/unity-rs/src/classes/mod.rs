pub mod class_id_type_to_class_map;
pub mod generated;
/// Unity object classes module
///
/// This module contains the base Object trait and implementations for
/// various Unity object types (Texture2D, Mesh, Material, etc.)
pub mod object;
pub mod pptr;
pub mod unknown_object;

// Re-export the main traits for convenient access
pub use object::Object;
pub use pptr::PPtr;
pub use unknown_object::UnknownObject;
