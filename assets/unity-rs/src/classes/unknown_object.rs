use super::object::{Object, ObjectReaderTrait};
use crate::helpers::type_tree_node::TypeTreeNode;
use serde_json::{Map, Value};
use std::{any::Any, fmt};

#[derive(Debug)]
pub struct UnknownObject {
    /// The TypeTreeNode that describes this object's structure
    node: Option<TypeTreeNode>,

    /// Dynamic fields stored as JSON values
    /// Equivalent to Python's __dict__
    data: Map<String, Value>,

    /// Link back to the ObjectReader (for save operations)
    object_reader: Option<Box<dyn ObjectReaderTrait>>,
}

impl Default for UnknownObject {
    fn default() -> Self {
        Self {
            node: None,
            data: Map::new(),
            object_reader: None,
        }
    }
}

impl UnknownObject {
    pub fn new(node: Option<TypeTreeNode>, data: Map<String, Value>) -> Self {
        Self {
            node,
            data,
            object_reader: None,
        }
    }

    // Convenience method for TypeTreeHelper
    pub fn from_map(node: TypeTreeNode, fields: Map<String, Value>) -> Self {
        Self::new(Some(node), fields)
    }
}

impl UnknownObject {
    pub fn get_type(&self) -> Option<&str> {
        self.node.as_ref().map(|n| n.m_type.as_str())
    }

    /// Converts this UnknownObject to a JSON value
    ///
    /// Returns the internal data map as a JSON object.
    /// This is equivalent to accessing Python's obj.__dict__
    ///
    /// # Returns
    ///
    /// A `serde_json::Value::Object` containing all fields
    ///
    /// Python equivalent: accessing __dict__ or using asdict()
    pub fn to_json(&self) -> Value {
        Value::Object(self.data.clone())
    }
}

impl Object for UnknownObject {
    fn object_reader(&self) -> Option<&dyn ObjectReaderTrait> {
        self.object_reader.as_ref().map(|r| r.as_ref())
    }

    fn set_object_reader(&mut self, reader: Box<dyn ObjectReaderTrait>) {
        self.object_reader = Some(reader);
    }

    fn type_name(&self) -> &str {
        self.get_type().unwrap_or("UnknownObject")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_json_value(&self) -> serde_json::Value {
        self.to_json()
    }
}

impl fmt::Display for UnknownObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = self.get_type().unwrap_or("Unknown");
        write!(f, "<UnknownObject<{}> ", type_str)?;

        let fields: Vec<String> = self
            .data
            .iter()
            .map(|(k, v)| {
                let v_str = format!("{:?}", v);
                let truncated = if v_str.len() > 100 {
                    format!("{}...", &v_str[..97])
                } else {
                    v_str
                };
                format!("{}={}", k, truncated)
            })
            .collect();

        write!(f, "{}>", fields.join(", "))
    }
}
