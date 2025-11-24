//! Error types for UnityRs
//!
//! This module defines custom error types that match Python UnityPy's exceptions.
//!
//! Python equivalent: UnityPy/exceptions.py

use thiserror::Error;

/// Main error type for UnityRs operations
///
/// This enum wraps all possible errors that can occur in UnityRs.
/// Using an enum allows us to handle different error types uniformly.
#[derive(Error, Debug)]
pub enum UnityError {
    /// TypeTree error - occurs when TypeTree nodes are missing or invalid
    ///
    /// Python equivalent: TypeTreeError
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityError;
    ///
    /// let err = UnityError::TypeTreeError {
    ///     message: "There are no TypeTree nodes for this object.".to_string(),
    ///     nodes: None,
    /// };
    /// ```
    #[error("TypeTree error: {message}")]
    TypeTreeError {
        /// The error message explaining what went wrong
        message: String,

        /// Optional TypeTree nodes for debugging
        /// In Python this is required, but in Rust we make it optional
        /// to allow more flexible error creation
        nodes: Option<Vec<String>>,
    },

    /// Unity version fallback error - occurs when no valid Unity version is found
    /// and the fallback version is not configured
    ///
    /// Python equivalent: UnityVersionFallbackError
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityError;
    ///
    /// let err = UnityError::UnityVersionFallbackError(
    ///     "No valid Unity version found, and fallback not configured".to_string()
    /// );
    /// ```
    #[error("Unity version fallback error: {0}")]
    UnityVersionFallbackError(String),

    /// I/O error wrapper
    ///
    /// This allows us to convert std::io::Error into UnityError automatically
    /// using the `?` operator
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityError;
    /// use std::io;
    ///
    /// fn read_file() -> Result<String, UnityError> {
    ///     std::fs::read_to_string("file.txt")?;  // Auto-converts io::Error to UnityError
    ///     Ok("success".to_string())
    /// }
    /// ```
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error for string messages
    ///
    /// Use this when you have a simple error message and don't need
    /// a specific error type
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityError;
    ///
    /// let err = UnityError::Other("Something went wrong".to_string());
    /// ```
    #[error("{0}")]
    Other(String),
}

/// Warning types for UnityRs operations
///
/// Warnings are non-fatal issues that the user should be aware of.
/// Unlike errors, warnings don't stop execution.
///
/// Python equivalent: UserWarning subclasses in exceptions.py
#[derive(Debug, Clone)]
pub enum UnityWarning {
    /// Unity version fallback warning - occurs when no valid Unity version is found
    /// but a fallback version is configured, so processing can continue
    ///
    /// Python equivalent: UnityVersionFallbackWarning
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityWarning;
    ///
    /// let warning = UnityWarning::version_fallback_warning(
    ///     "No valid Unity version found, defaulting to fallback version: 2019.4.0f1"
    /// );
    /// warning.emit();
    /// ```
    UnityVersionFallbackWarning(String),
}

impl std::fmt::Display for UnityWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnityWarning::UnityVersionFallbackWarning(msg) => {
                write!(f, "Unity version fallback warning: {}", msg)
            }
        }
    }
}

impl UnityWarning {
    /// Creates a UnityVersionFallbackWarning with a message
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityWarning;
    ///
    /// let warning = UnityWarning::version_fallback_warning("No version found");
    /// ```
    pub fn version_fallback_warning(message: impl Into<String>) -> Self {
        UnityWarning::UnityVersionFallbackWarning(message.into())
    }

    /// Emits the warning using the log crate
    ///
    /// This is the equivalent of Python's `warnings.warn()`
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityWarning;
    ///
    /// let warning = UnityWarning::version_fallback_warning("Using fallback");
    /// warning.emit();  // Logs the warning
    /// ```
    pub fn emit(&self) {
        log::warn!("{}", self);
    }
}

/// Convenience constructors for UnityError
impl UnityError {
    /// Creates a TypeTreeError with a message
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityError;
    ///
    /// let err = UnityError::type_tree_error("No nodes found", None);
    /// ```
    pub fn type_tree_error(message: impl Into<String>, nodes: Option<Vec<String>>) -> Self {
        UnityError::TypeTreeError {
            message: message.into(),
            nodes,
        }
    }

    /// Creates a UnityVersionFallbackError with a message
    ///
    /// # Example
    /// ```
    /// use unity_rs::errors::UnityError;
    ///
    /// let err = UnityError::version_fallback_error("No version configured");
    /// ```
    pub fn version_fallback_error(message: impl Into<String>) -> Self {
        UnityError::UnityVersionFallbackError(message.into())
    }
}

/// Type alias for Results using UnityError
///
/// This makes function signatures cleaner:
/// Instead of: `fn foo() -> Result<T, UnityError>`
/// You can write: `fn foo() -> UnityResult<T>`
pub type UnityResult<T> = Result<T, UnityError>;
