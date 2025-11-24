use serde::{Deserialize, Serialize};
use std::fmt;

/// Trait for ObjectReader to work with Object trait objects
///
/// The concrete ObjectReader<T> struct must implement this trait
pub trait ObjectReaderTrait: fmt::Debug {
    /// Returns a reference to the parent SerializedFile
    fn assets_file(&self) -> Option<&crate::files::serialized_file::SerializedFile>;

    /// Saves the object's typetree data back to the file
    fn save_typetree(&self, obj: &dyn Object) -> Result<(), String>;

    /// Returns a reference to self as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Base trait for all Unity objects
///
/// Python equivalent: Object ABC (Abstract Base Class)
pub trait Object: fmt::Debug {
    /// Gets a reference to the ObjectReader if set
    fn object_reader(&self) -> Option<&dyn ObjectReaderTrait>;

    /// Sets the ObjectReader for this object
    fn set_object_reader(&mut self, reader: Box<dyn ObjectReaderTrait>);

    /// Gets a reference to the parent SerializedFile
    fn assets_file(&self) -> Option<&crate::files::serialized_file::SerializedFile> {
        self.object_reader()?.assets_file()
    }

    /// Saves this object back to its source file
    fn save(&mut self) -> Result<(), String>
    where
        Self: Sized,
    {
        let reader = self
            .object_reader()
            .ok_or_else(|| "ObjectReader not set".to_string())?;
        reader.save_typetree(self as &dyn Object)
    }

    /// Returns the type name for display purposes
    fn type_name(&self) -> &str;

    /// Returns a reference to self as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Converts this object to a JSON value for serialization
    ///
    /// Used when writing typed objects back to binary format.
    /// Implemented by the impl_object! macro for all generated structs.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` representing the object's fields
    fn to_json_value(&self) -> serde_json::Value;
}

impl fmt::Display for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.type_name())
    }
}

/// Concrete Object struct for use in PPtrData<ObjectInfo>
///
/// Python equivalent: Object class when used as PPtr[Object]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObjectInfo {
    // Empty struct for now - just a marker type
    // The object_reader will be managed separately when needed
}

impl ObjectInfo {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ObjectInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Object for ObjectInfo {
    fn object_reader(&self) -> Option<&dyn ObjectReaderTrait> {
        None // ObjectInfo doesn't store object_reader
    }

    fn set_object_reader(&mut self, _reader: Box<dyn ObjectReaderTrait>) {
        // No-op for ObjectInfo marker type
    }

    fn type_name(&self) -> &str {
        "Object"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}
