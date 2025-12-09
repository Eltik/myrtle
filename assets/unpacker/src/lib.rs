pub mod collect_models;
pub mod collect_voice;
pub mod combine_rgb;
pub mod config;
pub mod decode_textasset;
pub mod flatbuffers_decode;
pub mod generated_fbs;
pub mod generated_fbs_yostar;
pub mod resolve_ab;
pub mod resource_manifest;
pub mod sprite_packer;
pub mod utils;

// FlatBuffer JSON serialization (handles unknown enum values gracefully)
#[macro_use]
pub mod fb_json_macros;
// Auto-generated implementations for ALL FlatBuffer types
pub mod fb_json_auto;
// Auto-generated implementations for Yostar-specific FlatBuffer types
pub mod fb_json_auto_yostar;
