// Clippy allows for complex codebase
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::regex_creation_in_loops)]
#![allow(clippy::uninlined_format_args)]

pub mod downloader;
pub mod utils;

// S3-compatible storage support
pub mod s3_bridge;
pub mod s3_manifest;
