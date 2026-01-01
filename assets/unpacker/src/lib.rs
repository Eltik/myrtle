// Clippy allows for complex codebase
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::manual_strip)]
#![allow(clippy::redundant_guards)]
#![allow(clippy::search_is_some)]
#![allow(clippy::uninlined_format_args)]

pub mod collect_models;
pub mod collect_voice;
pub mod combine_rgb;
pub mod config;
pub mod decode_textasset;
pub mod flatbuffers_decode;
pub mod generated_fbs;
pub mod generated_fbs_yostar;
pub mod reorganize;
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
