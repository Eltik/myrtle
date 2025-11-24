use crate::files::bundle_file::FileType;
use crate::files::ObjectReader;
use crate::helpers::import_helper::{check_file_type, find_sensitive_path, parse_file, FileSource};
use crate::helpers::type_tree_generator::TypeTreeGenerator;
use crate::streams::endian::Endian;
use crate::streams::endian_reader::MemoryReader;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use zip::ZipArchive;

/// Simplifies a file name for case-insensitive comparison
/// - Removes the path (keeps only basename)
/// - Keeps the extension (Python's ntpath.basename keeps it despite misleading docstring)
/// - Converts to lowercase
///
/// Example: "/path/to/File.ASSET" -> "file.asset"
///
/// Python equivalent: simplify_name() in environment.py line 343-349
pub fn simplify_name(name: &str) -> String {
    Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(name)
        .to_lowercase()
}

/// Environment manages the registry of all loaded files and CAB files.
///
/// This is critical for:
/// - Cross-file asset references (PPtr resolution)
/// - Dependency loading (load_dependencies)
/// - CAB file registry (for external references)
/// - File loading from disk
/// - Directory traversal
///
/// Python equivalent: UnityPy/environment.py line 27-341
#[derive(Debug)]
pub struct Environment {
    /// Registry of all loaded files (by name)
    /// Python: line 28
    pub files: HashMap<String, Rc<RefCell<FileType>>>,

    /// Registry of CAB files (for cross-file dependency resolution)
    /// CAB = Cabinet file (Unity's term for dependency files)
    /// Python: line 29
    pub cabs: HashMap<String, Rc<RefCell<FileType>>>,

    /// Base path for file loading
    /// Python: line 30
    pub path: Option<String>,

    /// Cache of all files found in directory tree (full paths)
    /// Built lazily when find_file() needs it
    /// Python: line 31
    pub local_files: Vec<String>,

    /// Cache of simplified file names (for case-insensitive matching)
    /// Parallel array to local_files
    /// Python: line 32
    pub local_files_simple: Vec<String>,

    /// TypeTree generator for MonoBehaviour scripts
    /// Python: line 33
    pub typetree_generator: Option<RefCell<TypeTreeGenerator>>,
}

impl Environment {
    /// Creates a new empty Environment
    ///
    /// Python equivalent: __init__() line 35-41
    pub fn new() -> Self {
        Environment {
            files: HashMap::new(),
            cabs: HashMap::new(),
            path: None,
            local_files: Vec::new(),
            local_files_simple: Vec::new(),
            typetree_generator: None,
        }
    }

    /// Creates a new Environment with a base path
    pub fn with_path(path: String) -> Self {
        Environment {
            files: HashMap::new(),
            cabs: HashMap::new(),
            path: Some(path),
            local_files: Vec::new(),
            local_files_simple: Vec::new(),
            typetree_generator: None,
        }
    }

    /// Registers a CAB file for dependency resolution
    ///
    /// CAB files are referenced by other files for cross-file asset references.
    /// When a PPtr has m_FileID != 0, it references an external file registered here.
    ///
    /// Python equivalent: register_cab() line 237-248
    ///
    /// # Arguments
    /// * `name` - The name of the file (will be simplified for matching)
    /// * `file` - The file to register
    pub fn register_cab(&mut self, name: String, file: Rc<RefCell<FileType>>) {
        let simple_name = simplify_name(&name);
        self.cabs.insert(simple_name, file);
    }

    /// Gets a CAB file by name (case-insensitive)
    ///
    /// Python equivalent: get_cab() line 250-264
    ///
    /// # Arguments
    /// * `name` - The name to search for (will be simplified automatically)
    ///
    /// # Returns
    /// The registered CAB file, or None if not found
    pub fn get_cab(&self, name: &str) -> Option<Rc<RefCell<FileType>>> {
        let simple_name = simplify_name(name);
        self.cabs.get(&simple_name).cloned()
    }

    /// Finds a file in the environment
    ///
    /// This method implements the full Python logic:
    /// 1. Check CAB registry
    /// 2. Try loading from disk (self.path + name)
    /// 3. Build local_files cache if needed
    /// 4. Search local_files for exact match
    /// 5. Search for files ending with the name
    /// 6. Load and return the file
    ///
    /// Python equivalent: find_file() line 294-340
    ///
    /// # Arguments
    /// * `name` - The file name to find
    /// * `is_dependency` - Whether this is a dependency load
    ///
    /// # Returns
    /// The found file, or an error if not found
    pub fn find_file(
        &mut self,
        name: &str,
        is_dependency: bool,
    ) -> Result<Rc<RefCell<FileType>>, String> {
        // Step 1: Check CAB registry (line 310-313)
        let simple_name = simplify_name(name);
        if let Some(cab) = self.get_cab(&simple_name) {
            return Ok(cab);
        }

        // Step 2: Try loading from disk (line 314-316)
        if let Some(base_path) = &self.path {
            let fp = Path::new(base_path).join(name);
            if fp.exists() {
                return self
                    .load_file_from_path(&fp.to_string_lossy().to_string(), is_dependency)
                    .map_err(|e| e.to_string());
            }

            // Try case-insensitive path resolution (not in Python, but improves robustness)
            if let Some(sensitive_path) = find_sensitive_path(base_path, name) {
                return self
                    .load_file_from_path(&sensitive_path, is_dependency)
                    .map_err(|e| e.to_string());
            }
        }

        // Step 3: Build local_files cache if needed (line 318-324)
        if self.local_files.is_empty() {
            if let Some(base_path) = self.path.clone() {
                self.walk_directory(&base_path).map_err(|e| e.to_string())?;
            }
        }

        // Step 4: Search local_files for exact match (line 326-329)
        let fp = if self.local_files.contains(&name.to_string()) {
            name.to_string()
        } else if let Some(idx) = self
            .local_files_simple
            .iter()
            .position(|x| x == &simple_name)
        {
            self.local_files[idx].clone()
        }
        // Step 5: Search for files ending with the name (line 331-336)
        else if let Some(f) = self.local_files.iter().find(|f| f.ends_with(name)) {
            f.clone()
        } else if let Some(f) = self
            .local_files_simple
            .iter()
            .position(|f| f.ends_with(&simple_name))
        {
            self.local_files[f].clone()
        } else {
            // Step 6: Not found - return error (line 337-338)
            return Err(format!(
                "File {} not found in {}",
                name,
                self.path.as_deref().unwrap_or("environment")
            ));
        };

        // Step 7: Load and return (line 340)
        self.load_file_from_path(&fp, is_dependency)
            .map_err(|e| e.to_string())
    }

    /// Walks a directory tree and populates local_files cache
    ///
    /// Python equivalent: Part of find_file() line 318-324
    fn walk_directory(&mut self, path: &str) -> io::Result<()> {
        fn walk_recursive(
            dir: &Path,
            files: &mut Vec<String>,
            simple: &mut Vec<String>,
        ) -> io::Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        walk_recursive(&path, files, simple)?;
                    } else {
                        let path_str = path.to_string_lossy().to_string();
                        files.push(path_str.clone());
                        simple.push(simplify_name(&path_str));
                    }
                }
            }
            Ok(())
        }

        walk_recursive(
            Path::new(path),
            &mut self.local_files,
            &mut self.local_files_simple,
        )
    }

    /// Loads a file from a path into the environment
    ///
    /// Python equivalent: Part of load_file() line 99-157
    ///
    /// # Arguments
    /// * `file_path` - Path to the file
    /// * `is_dependency` - Whether this is a dependency
    ///
    /// # Returns
    /// The loaded file
    fn load_file_from_path(
        &mut self,
        file_path: &str,
        is_dependency: bool,
    ) -> io::Result<Rc<RefCell<FileType>>> {
        // Read file bytes
        let mut file = fs::File::open(file_path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Create reader
        let mut reader = MemoryReader::new(bytes, crate::streams::endian::Endian::Little, 0);

        // Check file type
        let file_type = check_file_type(&mut reader)?;

        // Get file name for registry
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(file_path)
            .to_string();

        // Parse file
        let parsed_file = parse_file(
            reader,
            file_name.clone(),
            Some(file_type),
            is_dependency,
            None, // No parent for now
        )?;

        // Wrap in Rc<RefCell<>>
        let file_rc = Rc::new(RefCell::new(parsed_file));

        // Register CAB
        self.register_cab(file_name.clone(), file_rc.clone());

        // Store in files registry
        self.files.insert(file_name, file_rc.clone());

        Ok(file_rc)
    }

    /// Loads all files (list) into the Environment and merges .split files for common usage.
    ///
    /// Python equivalent: load_files() line 68-70
    ///
    /// # Arguments
    /// * `files` - List of file paths to load
    pub fn load_files(&mut self, files: Vec<String>) -> io::Result<()> {
        self.load_assets(files)
    }

    /// Helper to load split files (.split0, .split1, etc.)
    ///
    /// Python equivalent: _load_split_file() line 88-97
    ///
    /// # Arguments
    /// * `basename` - The base path without .splitN extension
    ///
    /// # Returns
    /// Concatenated bytes from all split files
    fn load_split_file(&self, basename: &str) -> io::Result<Vec<u8>> {
        let mut result = Vec::new();

        for i in 0..999 {
            let item = format!("{}.split{}", basename, i);
            if Path::new(&item).exists() {
                let mut file = fs::File::open(&item)?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                result.extend(bytes);
            } else if i > 0 {
                break;
            }
        }

        Ok(result)
    }

    /// Load all assets from a list of files, merging split files automatically
    ///
    /// Python equivalent: load_assets() line 266-292
    ///
    /// # Arguments
    /// * `assets` - List of file paths to load
    pub fn load_assets(&mut self, assets: Vec<String>) -> io::Result<()> {
        use regex::Regex;
        let re_split = Regex::new(r"(.*?([^/\\]+?))\.split\d+").unwrap();

        let mut split_files = Vec::new();

        for path in assets {
            if let Some(captures) = re_split.captures(&path) {
                let basepath = captures.get(1).unwrap().as_str();

                if split_files.contains(&basepath.to_string()) {
                    continue;
                }

                split_files.push(basepath.to_string());
                let data = self.load_split_file(basepath)?;

                // Load the merged split file
                let mut reader = MemoryReader::new(data, crate::streams::endian::Endian::Little, 0);
                let file_type = check_file_type(&mut reader)?;
                let parsed_file =
                    parse_file(reader, basepath.to_string(), Some(file_type), false, None)?;
                let file_rc = Rc::new(RefCell::new(parsed_file));
                self.register_cab(basepath.to_string(), file_rc.clone());
                self.files.insert(basepath.to_string(), file_rc);
            } else {
                // Regular file
                let _ = self.load_file_from_path(&path, false);
            }
        }

        Ok(())
    }

    /// Returns a list of all objects in the Environment
    ///
    /// Python equivalent: objects property line 184-204
    pub fn objects(&self) -> Vec<ObjectReader<()>> {
        fn search(files: &HashMap<String, Rc<RefCell<FileType>>>) -> Vec<ObjectReader<()>> {
            let mut ret = Vec::new();

            for file_rc in files.values() {
                let file_ref = file_rc.borrow();
                match &*file_ref {
                    FileType::SerializedFile(sf_rc) => {
                        let serialized_file = sf_rc.borrow();
                        if !serialized_file.is_dependency {
                            for obj in serialized_file.objects.values() {
                                ret.push(obj.clone());
                            }
                        }
                    }
                    FileType::BundleFile(bundle) => {
                        // Recursively search bundle files
                        for inner_file_rc in bundle.files.values() {
                            let inner_ref = inner_file_rc.borrow();
                            match &*inner_ref {
                                FileType::SerializedFile(sf_rc) => {
                                    let sf = sf_rc.borrow();
                                    if !sf.is_dependency {
                                        for obj in sf.objects.values() {
                                            ret.push(obj.clone());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    FileType::WebFile(web) => {
                        // Recursively search web files
                        for inner_file_rc in web.files.values() {
                            let inner_ref = inner_file_rc.borrow();
                            match &*inner_ref {
                                FileType::SerializedFile(sf_rc) => {
                                    let sf = sf_rc.borrow();
                                    if !sf.is_dependency {
                                        for obj in sf.objects.values() {
                                            ret.push(obj.clone());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            ret
        }

        search(&self.files)
    }

    /// Returns a dictionary of all objects in the Environment by their container path
    ///
    /// Python equivalent: container property line 206-214
    pub fn container(&self) -> HashMap<String, ObjectReader<()>> {
        let mut result = HashMap::new();

        for file_rc in self.files.values() {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                FileType::SerializedFile(sf_rc) => {
                    let serialized_file = sf_rc.borrow();
                    if !serialized_file.is_dependency {
                        for (path, &path_id) in serialized_file.container.items() {
                            if let Some(obj) = serialized_file.objects.get(&path_id) {
                                result.insert(path.clone(), obj.clone());
                            }
                        }
                    }
                }
                FileType::BundleFile(bundle) => {
                    // Recursively get container from bundle
                    for inner_file_rc in bundle.files.values() {
                        let inner_ref = inner_file_rc.borrow();
                        match &*inner_ref {
                            FileType::SerializedFile(sf_rc) => {
                                let sf = sf_rc.borrow();
                                if !sf.is_dependency {
                                    for (path, &path_id) in sf.container.items() {
                                        if let Some(obj) = sf.objects.get(&path_id) {
                                            result.insert(path.clone(), obj.clone());
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                FileType::WebFile(web) => {
                    // Recursively get container from web file
                    for inner_file_rc in web.files.values() {
                        let inner_ref = inner_file_rc.borrow();
                        match &*inner_ref {
                            FileType::SerializedFile(sf_rc) => {
                                let sf = sf_rc.borrow();
                                if !sf.is_dependency {
                                    for (path, &path_id) in sf.container.items() {
                                        if let Some(obj) = sf.objects.get(&path_id) {
                                            result.insert(path.clone(), obj.clone());
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        result
    }

    /// Lists all assets / SerializedFiles within this environment
    ///
    /// Python equivalent: assets property line 216-232
    pub fn assets(&self) -> Vec<Rc<RefCell<FileType>>> {
        fn gen_all_asset_files(
            files: &HashMap<String, Rc<RefCell<FileType>>>,
            ret: &mut Vec<Rc<RefCell<FileType>>>,
        ) {
            for file_rc in files.values() {
                let file_ref = file_rc.borrow();
                match &*file_ref {
                    FileType::SerializedFile(sf_rc) => {
                        let sf = sf_rc.borrow();
                        if !sf.is_dependency {
                            drop(sf);
                            drop(file_ref);
                            ret.push(Rc::clone(file_rc));
                        }
                    }
                    FileType::BundleFile(bundle) => {
                        gen_all_asset_files(&bundle.files, ret);
                    }
                    FileType::WebFile(web) => {
                        gen_all_asset_files(&web.files, ret);
                    }
                    _ => {}
                }
            }
        }

        let mut result = Vec::new();
        gen_all_asset_files(&self.files, &mut result);
        result
    }

    /// Dictionary-like accessor for Environment attributes
    ///
    /// Python equivalent: get() line 234-235
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "path" => self.path.clone(),
            _ => None,
        }
    }

    /// Saves all changed assets
    ///
    /// Python equivalent: save() line 172-182
    ///
    /// # Arguments
    /// * `pack` - Compression type ("none" or "lz4")
    /// * `out_path` - Output directory path
    pub fn save(&self, pack: &str, out_path: &str) -> io::Result<()> {
        fs::create_dir_all(out_path)?;

        for (fname, fitem) in &self.files {
            let is_changed = {
                let file_ref = fitem.borrow();
                match &*file_ref {
                    FileType::SerializedFile(sf_rc) => sf_rc.borrow().is_changed,
                    FileType::BundleFile(bundle) => bundle.is_changed,
                    _ => false,
                }
            };

            if !is_changed {
                continue;
            }

            let out_file_path = Path::new(out_path).join(Path::new(fname).file_name().unwrap());

            let file_ref = fitem.borrow();
            match &*file_ref {
                FileType::SerializedFile(sf_rc) => {
                    let mut sf = sf_rc.borrow_mut();
                    let mut out = fs::File::create(out_file_path)?;
                    let data = sf.save()?;
                    out.write_all(&data)?;
                }
                FileType::BundleFile(_bundle) => {
                    drop(file_ref); // Drop mutable borrow
                    let file_ref = fitem.borrow(); // Get immutable borrow
                    if let FileType::BundleFile(bundle) = &*file_ref {
                        let mut out = fs::File::create(out_file_path)?;
                        let data = bundle.save(Some(pack))?;
                        out.write_all(&data)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Sets environment references on all loaded SerializedFiles
    ///
    /// This must be called after loading files to enable resource resolution.
    /// AudioClips and other assets that load external resources require this.
    ///
    /// # Arguments
    /// * `env_rc` - Rc<RefCell<>> wrapped reference to this Environment
    ///
    /// # Example
    /// ```ignore
    /// let mut env = Environment::new();
    /// env.load_file(FileSource::Path("file.unity3d".to_string()), None, false);
    /// let env_rc = Rc::new(RefCell::new(env));
    /// Environment::set_environment_references(&env_rc);
    /// ```
    pub fn set_environment_references(env_rc: &Rc<RefCell<Self>>) -> io::Result<()> {
        // Collect file references first to avoid holding env_ref borrow
        let file_rcs: Vec<_> = {
            let env_ref = env_rc.borrow();
            env_ref.files.values().cloned().collect()
        };

        fn set_on_file(file_rc: &Rc<RefCell<FileType>>, env_rc: &Rc<RefCell<Environment>>) {
            // Clone necessary data before borrowing mutably
            let (sf_rc_opt, inner_files_opt) = {
                let file_ref = file_rc.borrow();
                match &*file_ref {
                    FileType::SerializedFile(sf_rc) => (Some(Rc::clone(sf_rc)), None),
                    FileType::BundleFile(bundle) => (
                        None,
                        Some(bundle.files.values().cloned().collect::<Vec<_>>()),
                    ),
                    FileType::WebFile(web) => {
                        (None, Some(web.files.values().cloned().collect::<Vec<_>>()))
                    }
                    _ => (None, None),
                }
            }; // file_ref borrow dropped here

            // Now process without holding file_ref
            if let Some(sf_rc) = sf_rc_opt {
                let mut sf = sf_rc.borrow_mut();
                sf.environment = Some(Rc::clone(env_rc));
            } else if let Some(inner_files) = inner_files_opt {
                for inner_file_rc in inner_files {
                    set_on_file(&inner_file_rc, env_rc);
                }
            }
        }

        for file_rc in file_rcs {
            set_on_file(&file_rc, env_rc);
        }

        // After setting environment references, register all CAB files from bundles
        // (they weren't registered during initial loading because environment was None)
        Self::register_all_cabs(env_rc)?;

        Ok(())
    }

    /// Registers all CAB files from loaded bundles into the environment's CAB registry
    ///
    /// This should be called after set_environment_references() to register CAB resources
    /// that were skipped during initial bundle loading (because environment was None at that time).
    fn register_all_cabs(env_rc: &Rc<RefCell<Self>>) -> io::Result<()> {
        // Collect all bundle files first
        let bundle_files: Vec<_> = {
            let env_ref = env_rc.borrow();
            env_ref
                .files
                .values()
                .filter_map(|file_rc| {
                    let file_ref = file_rc.borrow();
                    match &*file_ref {
                        FileType::BundleFile(_) => Some(Rc::clone(file_rc)),
                        _ => None,
                    }
                })
                .collect()
        };

        // Register CABs from each bundle
        for bundle_file_rc in bundle_files {
            let inner_files: Vec<(String, Rc<RefCell<FileType>>)> = {
                let file_ref = bundle_file_rc.borrow();
                if let FileType::BundleFile(bundle) = &*file_ref {
                    bundle
                        .files
                        .iter()
                        .map(|(name, file_rc)| (name.clone(), Rc::clone(file_rc)))
                        .collect()
                } else {
                    vec![]
                }
            };

            // Register each file that should be a CAB
            for (name, file_rc) in inner_files {
                let should_register = {
                    let file_ref = file_rc.borrow();
                    matches!(&*file_ref, FileType::Raw(_) | FileType::SerializedFile(_))
                };

                if should_register {
                    env_rc.borrow_mut().register_cab(name, file_rc);
                }
            }
        }

        Ok(())
    }

    /// Loads multiple files from a folder
    ///
    /// Python equivalent: load_folder() line 72-78
    ///
    /// # Arguments
    /// * `path` - The folder path to load from
    pub fn load_folder(&mut self, path: &str) -> io::Result<()> {
        fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        collect_files(&path, files)?;
                    } else {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        let mut file_paths = Vec::new();
        collect_files(Path::new(path), &mut file_paths)?;

        for file_path in file_paths {
            let path_str = file_path.to_string_lossy().to_string();
            let _ = self.load_file_from_path(&path_str, false);
        }

        Ok(())
    }

    /// Loads a file into the environment
    ///
    /// Python equivalent: load_file() line 99-157
    ///
    /// # Arguments
    /// * `file` - File source (path string or bytes)
    /// * `name` - Optional name override for the file
    /// * `is_dependency` - Whether this is a dependency load
    ///
    /// # Returns
    /// The loaded file, or None if load failed
    pub fn load_file(
        &mut self,
        file: FileSource,
        name: Option<String>,
        is_dependency: bool,
    ) -> Option<Rc<RefCell<FileType>>> {
        let re_split = Regex::new(r"(.*?([^/\\]+?))\.split\d+").unwrap();

        let (file_data, file_name) = match file {
            FileSource::Path(path_str) => {
                // Check if this is a split file (line 110-115)
                if let Some(captures) = re_split.captures(&path_str) {
                    let basepath = captures.get(1).unwrap().as_str();
                    let data = match self.load_split_file(basepath) {
                        Ok(d) => d,
                        Err(e) => {
                            log::error!("Failed to load split file '{}': {}", basepath, e);
                            return None;
                        }
                    };
                    (data, name.unwrap_or_else(|| basepath.to_string()))
                } else {
                    let mut final_path = path_str.clone();
                    let file_name = name.clone().unwrap_or_else(|| path_str.clone());

                    // Check if file exists (line 118)
                    if !Path::new(&final_path).exists() {
                        // Try relative to self.path (line 120-121)
                        if !Path::new(&final_path).is_absolute() {
                            if let Some(base_path) = &self.path {
                                final_path = Path::new(base_path)
                                    .join(&final_path)
                                    .to_string_lossy()
                                    .to_string();
                            }
                        }

                        // Check for .split0 file (line 123-124)
                        let split0_path = format!("{}.split0", final_path);
                        if Path::new(&split0_path).exists() {
                            let data = match self.load_split_file(&final_path) {
                                Ok(d) => d,
                                Err(e) => {
                                    log::error!("Failed to load split0 file '{}': {}", final_path, e);
                                    return None;
                                }
                            };
                            return self.load_file(
                                FileSource::Bytes(data),
                                Some(file_name),
                                is_dependency,
                            );
                        }

                        // File doesn't exist (line 129-130)
                        if !Path::new(&final_path).exists() {
                            log::error!("File does not exist: {}", final_path);
                            return None;
                        }
                    }

                    // Read file bytes (line 131-132)
                    let mut file = match fs::File::open(&final_path) {
                        Ok(f) => f,
                        Err(e) => {
                            log::error!("Failed to open file '{}': {}", final_path, e);
                            return None;
                        }
                    };
                    let mut bytes = Vec::new();
                    if let Err(e) = file.read_to_end(&mut bytes) {
                        log::error!("Failed to read file '{}': {}", final_path, e);
                        return None;
                    }

                    (bytes, file_name)
                }
            }
            FileSource::Bytes(bytes) => {
                // Use provided name or generate one
                let file_name = name.unwrap_or_else(|| format!("memory_{}", self.files.len()));
                (bytes, file_name)
            }
        };

        let file_data_clone = file_data.clone();
        let mut reader = MemoryReader::new(file_data, Endian::Little, 0);
        let file_type = match check_file_type(&mut reader) {
            Ok(ft) => ft,
            Err(e) => {
                log::error!("Failed to check file type for '{}': {}", file_name, e);
                return None;
            }
        };

        if file_type == crate::enums::file_type::FileType::ZIP {
            if let Err(e) = self.load_zip_file(FileSource::Bytes(file_data_clone)) {
                log::error!("Failed to load ZIP file '{}': {}", file_name, e);
            }
            return None;
        }

        let parsed_file = match parse_file(
            reader,
            file_name.clone(),
            Some(file_type),
            is_dependency,
            None,
        ) {
            Ok(pf) => pf,
            Err(e) => {
                log::error!("Failed to parse file '{}': {}", file_name, e);
                return None;
            }
        };

        let file_rc = Rc::new(RefCell::new(parsed_file));
        self.register_cab(file_name.clone(), file_rc.clone());
        self.files.insert(file_name, file_rc.clone());

        Some(file_rc)
    }

    /// Loads all files from a ZIP archive into the environment
    ///
    /// Python equivalent: load_zip_file() line 159-170
    ///
    /// # Arguments
    /// * `file_source` - ZIP file source (path string or bytes)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn load_zip_file(&mut self, file_source: FileSource) -> io::Result<()> {
        let zip_data = match file_source {
            FileSource::Path(path_str) => {
                // Read ZIP file from disk (line 161-162)
                let mut file = fs::File::open(&path_str)?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                bytes
            }
            FileSource::Bytes(bytes) => {
                // Use bytes directly (line 163-164)
                bytes
            }
        };

        let cursor = Cursor::new(zip_data);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let file_names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .collect();

        let re_split = Regex::new(r"(.*?([^/\\]+?))\.split\d+").unwrap();
        let mut split_files = Vec::new();

        for file_name in file_names {
            // Check if this is a split file
            if let Some(captures) = re_split.captures(&file_name) {
                let basepath = captures.get(1).unwrap().as_str();

                if split_files.contains(&basepath.to_string()) {
                    continue;
                }

                split_files.push(basepath.to_string());

                let mut merged_data = Vec::new();
                for i in 0..999 {
                    let split_name = format!("{}.split{}", basepath, i);
                    if let Ok(mut zip_file) = archive.by_name(&split_name) {
                        zip_file.read_to_end(&mut merged_data)?;
                    } else if i > 0 {
                        break;
                    }
                }

                if !merged_data.is_empty() {
                    let _ = self.load_file(
                        FileSource::Bytes(merged_data),
                        Some(basepath.to_string()),
                        false,
                    );
                }
                continue;
            }

            // Read file data from ZIP (line 291 - open_f(path).read())
            if let Ok(mut zip_file) = archive.by_name(&file_name) {
                let mut data = Vec::new();
                zip_file.read_to_end(&mut data)?;

                // Load file via load_file (line 292)
                let _ = self.load_file(FileSource::Bytes(data), Some(file_name), false);
            }
        }

        Ok(())
    }

    /// Loads all files into the Environment
    ///
    /// Python equivalent: load() line 80-86
    ///
    /// # Arguments
    /// * `files` - List of file paths to load
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn load(&mut self, files: Vec<String>) -> io::Result<()> {
        for file_path in files {
            // Check if file exists (line 85 - if self.fs.exists(f))
            if !Path::new(&file_path).exists() {
                continue;
            }

            // Read file bytes (line 83 - self.fs.open(f, "rb"))
            let mut file = fs::File::open(&file_path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;

            // Get basename for the key (line 83 - ntpath.basename(f))
            let basename = Path::new(&file_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&file_path)
                .to_string();

            // Load file with basename as name (line 83 - self.load_file(...))
            // This will store it in self.files with basename as key
            let _ = self.load_file(FileSource::Bytes(bytes), Some(basename), false);
        }

        Ok(())
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
