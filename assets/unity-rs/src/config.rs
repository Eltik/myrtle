//! Configuration module for UnityRs
//!
//! This module manages global configuration settings that affect how Unity files are parsed.
//!
//! Python equivalent: UnityPy/config.py

use crate::errors::{UnityError, UnityResult, UnityWarning};
use once_cell::sync::Lazy;
use std::sync::RwLock;

/// Global fallback Unity version
///
/// This is used when no version is defined by the SerializedFile or its BundleFile.
/// You may manually configure this value to a version string, e.g. "2.5.0f5".
///
/// Python equivalent: FALLBACK_UNITY_VERSION
///
/// # Example
/// ```
/// use unity_rs::config;
///
/// // Set the fallback version
/// config::set_fallback_unity_version(Some("2019.4.0f1".to_string()));
///
/// // Get the fallback version
/// match config::get_fallback_version() {
///     Ok(version) => println!("Using fallback: {}", version),
///     Err(e) => println!("No fallback configured: {}", e),
/// }
/// ```
static FALLBACK_UNITY_VERSION: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

/// Global setting for TypeTree parsing
///
/// Determines if the typetree structures for the Object types will be parsed.
/// Disabling this will reduce the load time by a lot (half of the time is spent on parsing the typetrees),
/// but it will also prevent saving an edited file.
///
/// Python equivalent: SERIALIZED_FILE_PARSE_TYPETREE
///
/// # Example
/// ```
/// use unity_rs::config;
///
/// // Disable typetree parsing for faster loading
/// config::set_parse_typetree(false);
///
/// // Check if typetree parsing is enabled
/// if config::should_parse_typetree() {
///     println!("TypeTree parsing is enabled");
/// }
/// ```
static SERIALIZED_FILE_PARSE_TYPETREE: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(true));

/// Gets the fallback Unity version
///
/// This function retrieves the configured fallback version. If no version is configured,
/// it returns an error. It also logs a warning when the fallback is used.
///
/// Python equivalent: get_fallback_version()
///
/// # Returns
/// - `Ok(String)` - The configured fallback version
/// - `Err(UnityError)` - If no fallback version is configured
///
/// # Example
/// ```
/// use unity_rs::config;
///
/// config::set_fallback_unity_version(Some("2019.4.0f1".to_string()));
///
/// match config::get_fallback_version() {
///     Ok(version) => println!("Fallback version: {}", version),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn get_fallback_version() -> UnityResult<String> {
    let version_guard = FALLBACK_UNITY_VERSION
        .read()
        .map_err(|e| UnityError::Other(format!("Failed to read fallback version: {}", e)))?;

    match version_guard.as_ref() {
        Some(version) => {
            // Log warning (equivalent to Python's warnings.warn)
            UnityWarning::version_fallback_warning(format!(
                "No valid Unity version found, defaulting to fallback version: {}",
                version
            ))
            .emit();
            Ok(version.clone())
        }
        None => Err(UnityError::version_fallback_error(
            "No valid Unity version found, and the fallback version is not correctly configured. \
            Please explicitly set the value using config::set_fallback_unity_version().",
        )),
    }
}

/// Sets the fallback Unity version
///
/// # Arguments
/// * `version` - The Unity version to use as fallback (e.g., "2019.4.0f1"), or `None` to clear
///
/// # Example
/// ```
/// use unity_rs::config;
///
/// config::set_fallback_unity_version(Some("2019.4.0f1".to_string()));
/// ```
pub fn set_fallback_unity_version(version: Option<String>) {
    if let Ok(mut guard) = FALLBACK_UNITY_VERSION.write() {
        *guard = version;
    }
}

/// Gets the current fallback version without logging a warning
///
/// This is useful for checking if a fallback is configured without triggering
/// the warning that `get_fallback_version()` produces.
///
/// # Returns
/// `Some(String)` if a fallback is configured, `None` otherwise
///
/// # Example
/// ```
/// use unity_rs::config;
///
/// if let Some(version) = config::get_fallback_version_silent() {
///     println!("Fallback is configured: {}", version);
/// }
/// ```
pub fn get_fallback_version_silent() -> Option<String> {
    FALLBACK_UNITY_VERSION
        .read()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Checks if TypeTree structures should be parsed
///
/// Returns `true` if TypeTree parsing is enabled (default), `false` otherwise.
///
/// Python equivalent: Checking SERIALIZED_FILE_PARSE_TYPETREE
///
/// # Example
/// ```
/// use unity_rs::config;
///
/// if config::should_parse_typetree() {
///     println!("TypeTree parsing is enabled");
/// }
/// ```
pub fn should_parse_typetree() -> bool {
    SERIALIZED_FILE_PARSE_TYPETREE
        .read()
        .map(|guard| *guard)
        .unwrap_or(true) // Default to true if lock is poisoned
}

/// Sets whether TypeTree structures should be parsed
///
/// # Arguments
/// * `parse` - `true` to enable TypeTree parsing, `false` to disable
///
/// # Example
/// ```
/// use unity_rs::config;
///
/// // Disable TypeTree parsing for faster loading (but can't save)
/// config::set_parse_typetree(false);
///
/// // Re-enable TypeTree parsing
/// config::set_parse_typetree(true);
/// ```
pub fn set_parse_typetree(parse: bool) {
    if let Ok(mut guard) = SERIALIZED_FILE_PARSE_TYPETREE.write() {
        *guard = parse;
    }
}
