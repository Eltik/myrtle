/// Asset extractor for Unity files
///
/// This module provides tools for extracting assets from Unity asset files,
/// matching the functionality of UnityPy's tools.extractor module.
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::enums::class_id_type::ClassIDType;
use crate::helpers::import_helper::FileSource;
use crate::Environment;

/// Global storage for MonoBehaviour TypeTree definitions
/// Maps "Assembly-Name.dll" -> "Class-Name" -> TypeTree nodes
pub static MONOBEHAVIOUR_TYPETREES: once_cell::sync::Lazy<
    RwLock<HashMap<String, HashMap<String, Vec<serde_json::Value>>>>,
> = once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

/// Extracts assets from Unity file(s) to a destination directory.
///
/// This function matches the Python `extract_assets()` function, providing
/// batch extraction of Unity assets with optional filtering and organization.
///
/// # Arguments
///
/// * `src` - Source file or folder path
/// * `dst` - Destination directory path
/// * `use_container` - If true, use container paths for organizing output
///
/// # Returns
///
/// Number of files extracted
///
/// # Example
///
/// ```no_run
/// use unity_rs::tools::extract_assets;
///
/// let extracted = extract_assets(
///     "assets.unity3d",
///     "output/",
///     true
/// ).unwrap();
/// println!("Extracted {} files", extracted);
/// ```
pub fn extract_assets<P: AsRef<Path>, S: AsRef<Path>>(
    src: S,
    dst: P,
    use_container: bool,
) -> io::Result<usize> {
    let src_path = src.as_ref();
    let mut env = Environment::new();

    // Load source - handle both files and directories
    if src_path.is_dir() {
        env.load_folder(
            src_path.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path")
            })?,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    } else {
        env.load_file(
            FileSource::Path(src_path.to_string_lossy().to_string()),
            None,
            false,
        )
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Failed to load source"))?;
    }

    if cfg!(debug_assertions) {
        eprintln!("Loaded {} files from environment", env.files.len());
    }

    // Set environment references on all SerializedFiles for resource resolution
    let env_rc = std::rc::Rc::new(std::cell::RefCell::new(env));
    Environment::set_environment_references(&env_rc)?;

    let dst = dst.as_ref();
    fs::create_dir_all(dst)?;

    let mut file_count = 0;

    if use_container {
        // Use container organization
        // Collect container entries first to avoid holding env borrow
        let container = {
            let env = env_rc.borrow();
            env.container()
        };

        if cfg!(debug_assertions) {
            eprintln!("Container has {} entries", container.len());
        }
        for (obj_path, obj) in container {
            if let Some(output_path) = extract_object(&obj, dst, Some(&obj_path))? {
                file_count += 1;
                if cfg!(debug_assertions) {
                    eprintln!("Extracted: {}", output_path.display());
                }
            }
        }
    } else {
        // Extract all objects without container organization
        // Collect all objects first to avoid borrow checker issues
        let mut objects_to_extract = Vec::new();

        {
            let env = env_rc.borrow();
            for (_name, file_rc) in &env.files {
                let file_ref = file_rc.borrow();
                match &*file_ref {
                    crate::files::bundle_file::FileType::SerializedFile(sf_rc) => {
                        let sf = sf_rc.borrow();
                        for (_id, obj) in &sf.objects {
                            objects_to_extract.push(obj.clone());
                        }
                    }
                    crate::files::bundle_file::FileType::BundleFile(bundle) => {
                        // Handle SerializedFiles inside BundleFiles
                        for (_bundle_name, bundle_file_rc) in &bundle.files {
                            let bundle_file_ref = bundle_file_rc.borrow();
                            if let crate::files::bundle_file::FileType::SerializedFile(sf_rc) =
                                &*bundle_file_ref
                            {
                                let sf = sf_rc.borrow();
                                for (_id, obj) in &sf.objects {
                                    objects_to_extract.push(obj.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        } // env borrow dropped here

        // Now extract all collected objects
        for obj in objects_to_extract {
            if let Some(output_path) = extract_object(&obj, dst, None)? {
                file_count += 1;
                if cfg!(debug_assertions) {
                    eprintln!("Extracted: {}", output_path.display());
                }
            }
        }
    }

    Ok(file_count)
}

/// Extract a single object to disk
///
/// Returns the output path if the object was extracted, None if skipped
fn extract_object(
    obj: &crate::files::object_reader::ObjectReader<()>,
    dst: &Path,
    container_path: Option<&str>,
) -> io::Result<Option<PathBuf>> {
    // Determine output path
    let output_path = if let Some(cpath) = container_path {
        // Use container path structure
        let parts: Vec<&str> = cpath.split('/').filter(|s| !s.is_empty()).collect();
        dst.join(parts.join("/"))
    } else {
        // Use object name
        let name = get_object_name(obj);
        dst.join(format!("{}_{}", name, obj.path_id))
    };

    // Create parent directory
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Export based on type
    let result = match obj.obj_type {
        ClassIDType::Texture2D => export_texture2d(obj, &output_path),
        ClassIDType::Cubemap => export_cubemap(obj, &output_path),
        ClassIDType::Texture3D => export_texture3d(obj, &output_path),
        ClassIDType::RenderTexture => export_rendertexture(obj, &output_path),
        ClassIDType::Sprite => export_sprite(obj, &output_path),
        ClassIDType::Mesh => export_mesh(obj, &output_path),
        ClassIDType::AudioClip => export_audioclip(obj, &output_path),
        ClassIDType::Shader => export_shader(obj, &output_path),
        ClassIDType::TextAsset => export_textasset(obj, &output_path),
        ClassIDType::Font => export_font(obj, &output_path),
        ClassIDType::Material => export_material_standalone(obj, &output_path),
        ClassIDType::MonoScript => export_monoscript(obj, &output_path),
        ClassIDType::AnimationClip => export_animationclip(obj, &output_path),
        ClassIDType::VideoClip => export_videoclip(obj, &output_path),
        ClassIDType::MovieTexture => export_movietexture(obj, &output_path),
        ClassIDType::MonoBehaviour => export_monobehaviour(obj, &output_path),
        ClassIDType::SpriteAtlas => export_spriteatlas(obj, &output_path),
        _ => {
            // Check for SpriteAtlas by numeric ClassID (65007) for Unity versions that use different IDs
            if obj.class_id as u32 == 65007 {
                export_spriteatlas(obj, &output_path)
            } else {
                Ok(None) // Skip unsupported types
            }
        }
    };

    // Handle errors gracefully - log and continue
    match result {
        Ok(path) => Ok(path),
        Err(e) => {
            if cfg!(debug_assertions) {
                if let Some(cpath) = container_path {
                    eprintln!(
                        "Failed to export {:?} at '{}' (PathID: {}): {}",
                        obj.obj_type, cpath, obj.path_id, e
                    );
                } else {
                    eprintln!(
                        "Failed to export {:?} ({}): {}",
                        obj.obj_type, obj.path_id, e
                    );
                }
            }
            Ok(None) // Skip failed exports instead of stopping
        }
    }
}

fn export_texture2d(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Texture2D;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to Texture2D struct
    let mut texture: Texture2D = match serde_json::from_value(data.clone()) {
        Ok(t) => t,
        Err(e) => {
            // Deserialization failed - try to extract image data from JSON
            if cfg!(debug_assertions) {
                eprintln!(
                    "Texture2D deserialize error (attempting raw data extraction): {}",
                    e
                );
            }

            // Try to extract image_data from the raw JSON
            if let Some(image_data_val) = data.get("image_data") {
                // Check if it's a byte array
                if let Some(bytes) = image_data_val.as_array() {
                    let image_bytes: Result<Vec<u8>, _> = bytes
                        .iter()
                        .map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                        .collect::<Option<Vec<u8>>>()
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidData, "Invalid image data")
                        });

                    if let Ok(img_bytes) = image_bytes {
                        // Try to detect format and export
                        if !img_bytes.is_empty() {
                            let output = path.with_extension("dds");
                            let mut file = fs::File::create(&output)?;
                            file.write_all(&img_bytes)?;
                            return Ok(Some(output));
                        }
                    }
                }
            }

            // If we can't extract image data, export as JSON
            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    let width = texture.m_Width.unwrap_or(0);
    let height = texture.m_Height.unwrap_or(0);

    if width == 0 || height == 0 {
        return Ok(None); // Skip empty textures
    }

    // Set the object_reader so the texture can access resource data
    texture.object_reader = Some(Box::new(obj.clone()));

    // Use save_texture_as_png which handles both embedded and StreamData
    let output = path.with_extension("png");
    crate::export::texture_2d_converter::save_texture_as_png(&texture, &output, true)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    Ok(Some(output))
}

fn export_cubemap(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Cubemap;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let cubemap: Cubemap = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("Cubemap deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export cubemap data as raw binary (can be processed by other tools)
    if let Some(ref image_data) = cubemap.image_data {
        if !image_data.is_empty() {
            let output = path.with_extension("dds");
            let mut file = fs::File::create(&output)?;
            file.write_all(image_data)?;
            return Ok(Some(output));
        }
    }

    Ok(None)
}

fn export_texture3d(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Texture3D;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let texture3d: Texture3D = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("Texture3D deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export 3D texture data as raw binary
    if let Some(ref image_data) = texture3d.image_data {
        if !image_data.is_empty() {
            let output = path.with_extension("dds");
            let mut file = fs::File::create(&output)?;
            file.write_all(image_data)?;
            return Ok(Some(output));
        }
    }

    Ok(None)
}

fn export_rendertexture(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::RenderTexture;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let render_texture: RenderTexture = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("RenderTexture deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // RenderTexture is a render target, export metadata as JSON
    let json = serde_json::to_string_pretty(&render_texture)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_sprite(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Sprite;

    // Read sprite data
    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to Sprite struct
    let sprite: Sprite = match serde_json::from_value(data.clone()) {
        Ok(s) => s,
        Err(e) => {
            // Deserialization failed - export as JSON instead
            if cfg!(debug_assertions) {
                eprintln!(
                    "Sprite deserialize error (exporting as JSON fallback): {}",
                    e
                );
            }

            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    // Get the SerializedFile from the ObjectReader
    let assets_file = obj
        .assets_file
        .as_ref()
        .and_then(|weak| weak.upgrade())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Sprite's ObjectReader has no assets_file reference",
            )
        })?;

    // Export sprite to PNG (pass Rc directly so it can be borrowed as needed inside)
    let output = path.with_extension("png");
    crate::export::sprite_helper::save_sprite(
        &sprite,
        &assets_file,
        output
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid output path"))?,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    Ok(Some(output))
}

fn export_spriteatlas(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::{SpriteAtlas, Texture2D};

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to SpriteAtlas struct
    let atlas: SpriteAtlas = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("SpriteAtlas deserialize error: {}", e);
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Get the SerializedFile from the ObjectReader
    let assets_file = obj
        .assets_file
        .as_ref()
        .and_then(|weak| weak.upgrade())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "SpriteAtlas's ObjectReader has no assets_file reference",
            )
        })?;

    // Export the atlas's packed texture(s) from m_RenderDataMap
    if let Some(render_data_map) = &atlas.m_RenderDataMap {
        if !render_data_map.is_empty() {
            // Get the first texture from the render data map (most atlases have one packed texture)
            let first_atlas_data = &render_data_map[0].1;

            if let Some(ref texture_ptr) = first_atlas_data.texture {
                // Read the texture
                let assets_file_ref = assets_file.borrow();
                let mut texture = texture_ptr.read(&assets_file_ref).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to read atlas texture: {}", e),
                    )
                })?;

                // Set the object_reader so the texture can access StreamData
                if let Some(obj_reader) = assets_file_ref.objects.get(&texture_ptr.m_path_id) {
                    texture.object_reader = Some(Box::new(obj_reader.clone()));
                }
                drop(assets_file_ref);

                // Export as PNG with flip=true (standard vertical flip for Unity textures)
                let output = path.with_extension("png");
                crate::export::texture_2d_converter::save_texture_as_png(&texture, &output, true)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

                return Ok(Some(output));
            }
        }
    }

    // No texture to export
    Ok(None)
}

fn export_material_standalone(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Material;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let material: Material = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("Material deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export material to MTL format
    let mtl_content = crate::export::mesh_renderer_exporter::export_material(&material)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let output = path.with_extension("mtl");
    let mut file = fs::File::create(&output)?;
    file.write_all(mtl_content.as_bytes())?;

    Ok(Some(output))
}

fn export_monoscript(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::MonoScript;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let script: MonoScript = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("MonoScript deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // MonoScript is metadata about a C# script, not the actual source code
    // Export as JSON metadata
    let json = serde_json::to_string_pretty(&script)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_animationclip(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::AnimationClip;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let anim: AnimationClip = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("AnimationClip deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // Export AnimationClip as JSON with all curve data
    // This can be imported into other tools or processed further
    let json =
        serde_json::to_string_pretty(&anim).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_videoclip(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::VideoClip;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let video: VideoClip = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("VideoClip deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // VideoClip data is usually in external resources
    // Try to get the raw video data
    if let Ok(raw_data) = obj_clone.get_raw_data() {
        if !raw_data.is_empty() {
            // Export as .ogv (Ogg Theora) which is Unity's default video format
            // Or could be .mp4, .webm depending on format
            let output = path.with_extension("ogv");
            let mut file = fs::File::create(&output)?;
            file.write_all(&raw_data)?;
            return Ok(Some(output));
        }
    }

    // If no raw data, export metadata as JSON
    let json = serde_json::to_string_pretty(&video)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let output = path.with_extension("json");
    let mut file = fs::File::create(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(Some(output))
}

fn export_movietexture(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::MovieTexture;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let movie: MovieTexture = serde_json::from_value(data.clone()).map_err(|e| {
        if cfg!(debug_assertions) {
            eprintln!("MovieTexture deserialize error: {}", e);
            eprintln!(
                "JSON data: {}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            );
        }
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    // MovieTexture has embedded movie data
    if let Some(ref movie_data) = movie.m_MovieData {
        if !movie_data.is_empty() {
            // Export as .ogv (Ogg Theora) which is Unity's MovieTexture format
            let output = path.with_extension("ogv");
            let mut file = fs::File::create(&output)?;
            file.write_all(movie_data)?;
            return Ok(Some(output));
        }
    }

    Ok(None)
}

fn export_mesh(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Mesh;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to Mesh struct
    let mesh: Mesh = match serde_json::from_value(data.clone()) {
        Ok(m) => m,
        Err(e) => {
            // Deserialization failed - export as JSON instead
            if cfg!(debug_assertions) {
                eprintln!("Mesh deserialize error (exporting as JSON fallback): {}", e);
            }

            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    let obj_data = crate::export::mesh_exporter::export_mesh(&mesh, "obj")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let output = path.with_extension("obj");
    let mut file = fs::File::create(&output)?;
    file.write_all(obj_data.as_bytes())?;

    Ok(Some(output))
}

fn export_audioclip(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::AudioClip;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    // Try to deserialize to AudioClip struct
    let mut audio: AudioClip = match serde_json::from_value(data.clone()) {
        Ok(a) => a,
        Err(e) => {
            // Deserialization failed - try to extract audio data from JSON
            if cfg!(debug_assertions) {
                eprintln!(
                    "AudioClip deserialize error (attempting raw data extraction): {}",
                    e
                );
            }

            // Try to extract m_AudioData from the raw JSON
            if let Some(audio_data_val) = data.get("m_AudioData") {
                if let Some(bytes) = audio_data_val.as_array() {
                    let audio_bytes: Result<Vec<u8>, _> = bytes
                        .iter()
                        .map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                        .collect::<Option<Vec<u8>>>()
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidData, "Invalid audio data")
                        });

                    if let Ok(aud_bytes) = audio_bytes {
                        if !aud_bytes.is_empty() {
                            let output = path.with_extension("fsb");
                            let mut file = fs::File::create(&output)?;
                            file.write_all(&aud_bytes)?;
                            return Ok(Some(output));
                        }
                    }
                }
            }

            // If we can't extract audio data, export as JSON
            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let output = path.with_extension("json");
            let mut file = fs::File::create(&output)?;
            file.write_all(json.as_bytes())?;
            return Ok(Some(output));
        }
    };

    // Set the object_reader so the audio clip can access resource data
    audio.object_reader = Some(Box::new(obj.clone()));

    let samples = crate::export::audio_clip_converter::extract_audioclip_samples(&mut audio, true)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    if samples.is_empty() {
        return Ok(None);
    }

    if samples.len() == 1 {
        let output = path.with_extension("wav");
        let mut file = fs::File::create(&output)?;
        file.write_all(samples.values().next().unwrap())?;
        Ok(Some(output))
    } else {
        fs::create_dir_all(path)?;
        for (name, data) in &samples {
            let output = path.join(format!("{}.wav", name));
            let mut file = fs::File::create(&output)?;
            file.write_all(data)?;
        }
        Ok(Some(path.to_path_buf()))
    }
}

fn export_shader(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Shader;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    let shader: Shader =
        serde_json::from_value(data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let shader_text = crate::export::shader_converter::export_shader(&shader)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    let output = path.with_extension("txt");
    let mut file = fs::File::create(&output)?;
    file.write_all(shader_text.as_bytes())?;

    Ok(Some(output))
}

fn export_textasset(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::TextAsset;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    let text: TextAsset =
        serde_json::from_value(data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let script = text.m_Script.as_deref().unwrap_or("");
    if script.is_empty() {
        return Ok(None);
    }

    let output = path.with_extension("txt");
    let mut file = fs::File::create(&output)?;
    file.write_all(script.as_bytes())?;

    Ok(Some(output))
}

fn export_font(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    use crate::generated::Font;

    let mut obj_clone = obj.clone();
    let data = obj_clone
        .read(false)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    let font: Font =
        serde_json::from_value(data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    if let Some(ref font_data) = font.m_FontData {
        // Convert Vec<i8> to &[u8] for comparison
        let font_data_bytes: Vec<u8> = font_data.iter().map(|&b| b as u8).collect();
        let ext = if font_data_bytes.len() >= 4 && &font_data_bytes[0..4] == b"OTTO" {
            "otf"
        } else {
            "ttf"
        };

        let output = path.with_extension(ext);
        let mut file = fs::File::create(&output)?;
        file.write_all(&font_data_bytes)?;
        return Ok(Some(output));
    }

    Ok(None)
}

fn export_monobehaviour(
    obj: &crate::files::object_reader::ObjectReader<()>,
    path: &Path,
) -> io::Result<Option<PathBuf>> {
    // Try to export as JSON using TypeTree
    let mut obj_clone = obj.clone();
    if let Ok(data) = obj_clone.read_typetree(None, true, false) {
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let output = path.with_extension("json");
        let mut file = fs::File::create(&output)?;
        file.write_all(json.as_bytes())?;
        return Ok(Some(output));
    }

    // Fall back to raw binary
    if let Ok(raw_data) = obj_clone.get_raw_data() {
        let output = path.with_extension("bin");
        let mut file = fs::File::create(&output)?;
        file.write_all(&raw_data)?;
        return Ok(Some(output));
    }

    Ok(None)
}

fn get_object_name(obj: &crate::files::object_reader::ObjectReader<()>) -> String {
    let mut obj_clone = obj.clone();
    if let Ok(data) = obj_clone.read(false) {
        if let Some(name) = data.get("m_Name").and_then(|v| v.as_str()) {
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    format!("{:?}", obj.obj_type)
}

// Re-export for compatibility
pub use extract_assets as export_obj;
