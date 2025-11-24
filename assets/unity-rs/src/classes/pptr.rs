use crate::enums::class_id_type::ClassIDType;
use crate::files::bundle_file::FileType;
use crate::files::object_reader::ObjectReader;
use crate::files::serialized_file::SerializedFile;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct PPtr<'a, T> {
    pub m_file_id: i32,
    pub m_path_id: i64,
    pub assetsfile: Option<&'a SerializedFile>,
    _phantom: PhantomData<T>, // Zero-size type marker
}

impl<'a, T> PPtr<'a, T> {
    pub fn new(m_file_id: i32, m_path_id: i64) -> Self {
        Self {
            m_file_id,
            m_path_id,
            assetsfile: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_assetsfile(m_file_id: i32, m_path_id: i64, assetsfile: &'a SerializedFile) -> Self {
        Self {
            m_file_id,
            m_path_id,
            assetsfile: Some(assetsfile),
            _phantom: PhantomData,
        }
    }

    // Backwards compatibility accessors
    pub fn file_id(&self) -> i32 {
        self.m_file_id
    }

    pub fn path_id(&self) -> i64 {
        self.m_path_id
    }

    // Check if pointer is null
    pub fn is_null(&self) -> bool {
        self.m_path_id == 0
    }

    /// Dereferences the pointer to get the ObjectReader
    pub fn deref(
        &self,
        assetsfile_override: Option<&'a SerializedFile>,
    ) -> Result<ObjectReader<()>, String> {
        let assetsfile = assetsfile_override
            .or(self.assetsfile)
            .ok_or("PPtr can't deref without an assetsfile!")?;

        if self.m_path_id == 0 {
            return Err("PPtr can't deref with m_PathID == 0!".to_string());
        }

        let target_file: &SerializedFile = if self.m_file_id == 0 {
            // Same file
            assetsfile
        } else {
            let external_id = (self.m_file_id - 1) as usize;
            if external_id >= assetsfile.externals.len() {
                return Err("Failed to resolve pointer - invalid m_FileID!".to_string());
            }

            let external = &assetsfile.externals[external_id];
            let parent_weak = assetsfile
                .parent
                .as_ref()
                .ok_or_else(|| format!("PPtr points to {} but no parent is set!", external.path))?;

            let parent_rc = parent_weak
                .upgrade()
                .ok_or("Parent reference no longer valid".to_string())?;

            // Clean external path (Python lines 68-73)
            let mut clean_path = external.path.clone();
            if clean_path.starts_with("archive:/") {
                clean_path = clean_path[9..].to_string();
            }
            if clean_path.starts_with("assets/") {
                clean_path = clean_path[7..].to_string();
            }
            // Get basename and lowercase
            clean_path = clean_path
                .rsplit('/')
                .next()
                .unwrap_or(&clean_path)
                .to_lowercase();

            // Find file in parent (Python lines 75-87)
            let file_rc = parent_rc.borrow().find_file(&clean_path).ok_or_else(|| {
                format!("Failed to resolve pointer - {} not found!", external.path)
            })?;

            let borrowed = file_rc.borrow();
            match &*borrowed {
                FileType::SerializedFile(sf_rc) => {
                    let sf = sf_rc.borrow();
                    return Ok(sf
                        .objects
                        .get(&self.m_path_id)
                        .ok_or_else(|| format!("Object with PathID {} not found", self.m_path_id))?
                        .clone());
                }
                _ => {
                    return Err(format!(
                        "External file {} is not a SerializedFile",
                        external.path
                    ));
                }
            }
        };
        Ok(target_file
            .objects
            .get(&self.m_path_id)
            .ok_or_else(|| format!("Object with PathID {} not found", self.m_path_id))?
            .clone())
    }

    pub fn as_bool(&self) -> bool {
        self.m_path_id != 0
    }

    /// Gets the ClassIDType of the referenced object
    ///
    /// Python equivalent: lines 31-33
    pub fn get_type(
        &self,
        assetsfile_override: Option<&'a SerializedFile>,
    ) -> Result<ClassIDType, String> {
        Ok(self.deref(assetsfile_override)?.obj_type)
    }

    /// Reads and parses the object (backwards compatibility - UnityPy 1.x syntax)
    ///
    /// Python equivalent: lines 36-37
    pub fn read(
        &self,
        assetsfile_override: Option<&'a SerializedFile>,
    ) -> Result<serde_json::Value, String> {
        self.deref_parse_as_object(assetsfile_override)
    }

    /// Reads the object's typetree as dict (backwards compatibility - UnityPy 1.x syntax)
    ///
    /// Python equivalent: lines 40-41
    pub fn read_typetree(
        &self,
        assetsfile_override: Option<&'a SerializedFile>,
    ) -> Result<serde_json::Value, String> {
        self.deref_parse_as_dict(assetsfile_override)
    }

    /// Dereferences and parses the object as a Unity Object (UnityPy 2.x syntax)
    ///
    /// Python equivalent: lines 91-92
    pub fn deref_parse_as_object(
        &self,
        assetsfile_override: Option<&'a SerializedFile>,
    ) -> Result<serde_json::Value, String> {
        let mut obj_reader = self.deref(assetsfile_override)?;
        obj_reader.parse_as_object(None, true)
    }

    /// Dereferences and parses the object as a dictionary (UnityPy 2.x syntax)
    ///
    /// Python equivalent: lines 94-97
    pub fn deref_parse_as_dict(
        &self,
        assetsfile_override: Option<&'a SerializedFile>,
    ) -> Result<serde_json::Value, String> {
        let mut obj_reader = self.deref(assetsfile_override)?;
        obj_reader.parse_as_dict(None, true)
    }
}

impl<'a, T> Hash for PPtr<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.m_file_id.hash(state);
        self.m_path_id.hash(state);
    }
}

impl<'a, T> PartialEq for PPtr<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.m_file_id == other.m_file_id && self.m_path_id == other.m_path_id
    }
}

impl<'a, T> Eq for PPtr<'a, T> {} // Full equality

impl<'a, T> fmt::Display for PPtr<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PPtr<{}>(FileID: {}, PathID: {})",
            std::any::type_name::<T>(),
            self.m_file_id,
            self.m_path_id
        )
    }
}

/// Serializable PPtr for use in generated classes
///
/// This is a data-only version of PPtr that can be deserialized from JSON.
/// It doesn't hold a reference to the SerializedFile (no lifetime).
/// Can be converted to a full PPtr<'a, T> when you have a SerializedFile reference.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PPtrData<T> {
    #[serde(rename = "m_FileID")]
    pub m_file_id: i32,

    #[serde(rename = "m_PathID")]
    pub m_path_id: i64,

    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> PPtrData<T> {
    /// Creates a new PPtrData
    pub fn new(m_file_id: i32, m_path_id: i64) -> Self {
        Self {
            m_file_id,
            m_path_id,
            _phantom: PhantomData,
        }
    }

    /// Converts to a full PPtr with a SerializedFile reference
    pub fn to_pptr<'a>(&self, assetsfile: &'a SerializedFile) -> PPtr<'a, T> {
        PPtr::with_assetsfile(self.m_file_id, self.m_path_id, assetsfile)
    }

    /// Converts to a PPtr without a SerializedFile reference
    pub fn to_pptr_bare(&self) -> PPtr<'static, T> {
        PPtr::new(self.m_file_id, self.m_path_id)
    }
}

impl<T> PPtrData<T>
where
    T: serde::de::DeserializeOwned, // Requires T to be deserializable
{
    /// Reads and deserializes the referenced object
    ///
    /// Python equivalent: PPtr.read() (lines 36-37, 91-92)
    ///
    /// # Arguments
    /// * `assetsfile` - SerializedFile to resolve the pointer in
    ///
    /// # Returns
    /// The deserialized object of type T
    ///
    /// # Errors
    /// Returns error if:
    /// - PathID is 0 (null pointer)
    /// - Object not found in assets file
    /// - Deserialization fails
    ///
    /// # Example
    /// ```ignore
    /// let mesh_ptr: PPtrData<Mesh> = ...;
    /// let mesh: Mesh = mesh_ptr.read(assetsfile)?;
    /// ```
    pub fn read(&self, assetsfile: &SerializedFile) -> Result<T, String> {
        // Step 1: Check for null pointer
        if self.m_path_id == 0 {
            return Err("Cannot read null pointer (PathID is 0)".to_string());
        }

        // Step 2: Convert to PPtr with SerializedFile reference
        let pptr = self.to_pptr(assetsfile);

        // Step 3: Dereference and parse as JSON (Python: deref().parse_as_object())
        let json_value = pptr.deref_parse_as_object(None)?;

        // Step 4: Deserialize JSON into concrete type T
        serde_json::from_value(json_value)
            .map_err(|e| format!("Failed to deserialize object: {}", e))
    }

    /// Checks if the pointer is null (PathID is 0)
    ///
    /// Python equivalent: __bool__ (lines 99-100)
    pub fn is_null(&self) -> bool {
        self.m_path_id == 0
    }

    /// Attempts to read the object, returning None if pointer is null
    ///
    /// Convenience method that combines is_null() check with read()
    ///
    /// # Example
    /// ```ignore
    /// if let Some(mesh) = mesh_ptr.try_read(assetsfile)? {
    ///     // Use mesh
    /// }
    /// ```
    pub fn try_read(&self, assetsfile: &SerializedFile) -> Result<Option<T>, String> {
        if self.is_null() {
            Ok(None)
        } else {
            self.read(assetsfile).map(Some)
        }
    }
}

impl<T> Default for PPtrData<T> {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
