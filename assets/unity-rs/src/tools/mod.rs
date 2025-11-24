/// Tools module for Unity asset manipulation
///
/// This module provides utilities for extracting and processing Unity assets.
pub mod extractor;

// Re-export commonly used items
pub use extractor::{extract_assets, MONOBEHAVIOUR_TYPETREES};
