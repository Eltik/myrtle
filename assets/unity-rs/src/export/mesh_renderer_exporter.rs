//! Mesh renderer exporter for Unity MeshRenderer and SkinnedMeshRenderer
//!
//! Exports Unity Renderer objects to 3D formats with materials and textures.
//! Currently supports: OBJ + MTL + PNG textures
//!
//! Python equivalent: export/MeshRendererExporter.py (182 lines)

use crate::classes::generated::{
    Material, Mesh, MeshFilter, MeshRenderer, Renderer, SkinnedMeshRenderer, Texture2D, UnityTexEnv,
};
use crate::classes::object::Object;
use crate::classes::pptr::PPtrData;
use crate::enums::class_id_type::ClassIDType;
use crate::errors::{UnityError, UnityResult};
use crate::export::mesh_exporter::export_mesh_obj;
use crate::export::texture_2d_converter;
use crate::Color;
use std::collections::HashMap;
use std::path::Path;

/// Gets the mesh from a Renderer (MeshRenderer or SkinnedMeshRenderer)
///
/// Python equivalent: get_mesh (lines 25-45)
///
/// # Arguments
/// * `renderer` - Any object implementing Object trait (duck typing like Python)
///
/// # Returns
/// `Ok(Some(Mesh))` - The mesh if found
/// `Ok(None)` - No mesh found or not a renderer
/// `Err(UnityError)` - If reading fails
///
/// # Python Equivalent
/// ```python
/// def get_mesh(meshR: Renderer) -> Optional[Mesh]:
///     if isinstance(meshR, SkinnedMeshRenderer):
///         if meshR.m_Mesh:
///             return meshR.m_Mesh.read()
///     else:
///         m_GameObject = meshR.m_GameObject.read()
///         for comp in m_GameObject.m_Component:
///             # ... traverse to find MeshFilter
/// ```
pub fn get_mesh(renderer: &dyn Object) -> UnityResult<Option<Mesh>> {
    // STEP 1: Get ObjectReader to check actual type (Python's isinstance)
    // Python line 26: if isinstance(meshR, SkinnedMeshRenderer):
    let obj_reader = renderer
        .object_reader()
        .ok_or_else(|| UnityError::Other("Renderer has no object_reader".to_string()))?;

    // STEP 2: Get SerializedFile for reading PPtr references
    let assets_file = obj_reader
        .assets_file()
        .ok_or_else(|| UnityError::Other("ObjectReader has no SerializedFile".to_string()))?;

    // STEP 3: Check actual type and handle accordingly
    match renderer.type_name() {
        "SkinnedMeshRenderer" => {
            // Python lines 26-28: SkinnedMeshRenderer case
            // if isinstance(meshR, SkinnedMeshRenderer):
            //     if meshR.m_Mesh:
            //         return meshR.m_Mesh.read()

            // Downcast to SkinnedMeshRenderer (like Python's type narrowing after isinstance)
            let skinned = renderer
                .as_any()
                .downcast_ref::<SkinnedMeshRenderer>()
                .ok_or_else(|| {
                    UnityError::Other("Failed to downcast to SkinnedMeshRenderer".to_string())
                })?;

            // Python line 27: if meshR.m_Mesh:
            let mesh_ptr = skinned.m_Mesh.as_ref().ok_or_else(|| {
                UnityError::Other("SkinnedMeshRenderer has no m_Mesh".to_string())
            })?;

            if mesh_ptr.is_null() {
                return Ok(None);
            }

            // Python line 28: return meshR.m_Mesh.read()
            mesh_ptr
                .read(assets_file)
                .map(Some)
                .map_err(|e| UnityError::Other(e))
        }

        "MeshRenderer" => {
            // Python lines 29-45: MeshRenderer case
            // else:
            //     m_GameObject = meshR.m_GameObject.read()
            //     for comp in m_GameObject.m_Component:

            // Downcast to MeshRenderer
            let mesh_renderer = renderer
                .as_any()
                .downcast_ref::<MeshRenderer>()
                .ok_or_else(|| {
                    UnityError::Other("Failed to downcast to MeshRenderer".to_string())
                })?;

            // Python line 30: m_GameObject = meshR.m_GameObject.read()
            let game_object_ptr = mesh_renderer
                .m_GameObject
                .as_ref()
                .ok_or_else(|| UnityError::Other("MeshRenderer has no m_GameObject".to_string()))?;

            let game_object = game_object_ptr
                .read(assets_file)
                .map_err(|e| UnityError::Other(e))?;

            // Python lines 31-44: Iterate through components
            // for comp in m_GameObject.m_Component:
            let components = game_object
                .m_Component
                .as_ref()
                .ok_or_else(|| UnityError::Other("GameObject has no m_Component".to_string()))?;
            for (_comp_id, comp_pptr) in components {
                // Python lines 32-37: Handle tuple format and skip nulls
                // if isinstance(comp, tuple):
                //     pptr = comp[1]
                // else:
                //     pptr = comp.component
                // if not pptr:
                //     continue

                if comp_pptr.is_null() {
                    continue;
                }

                // Python line 38: obj = pptr.deref()
                // Get ObjectReader without deserializing yet
                let comp_reader = match comp_pptr.to_pptr(assets_file).deref(None) {
                    Ok(reader) => reader,
                    Err(_) => continue, // Skip if deref fails
                };

                // Python lines 39-40: if not obj: continue
                // (Already handled by deref result)

                // Python line 41: if obj.type.name == "MeshFilter":
                if comp_reader.obj_type == ClassIDType::MeshFilter {
                    // Python line 42: filter: MeshFilter = pptr.read()
                    // Create PPtrData<MeshFilter> from Component pointer
                    let filter_pptr =
                        PPtrData::<MeshFilter>::new(comp_pptr.m_file_id, comp_pptr.m_path_id);
                    let filter: MeshFilter = match filter_pptr.read(assets_file) {
                        Ok(f) => f,
                        Err(_) => continue, // Skip if read fails
                    };

                    // Python lines 43-44: if filter.m_Mesh: return filter.m_Mesh.read()
                    if let Some(ref mesh_pptr) = filter.m_Mesh {
                        return mesh_pptr
                            .try_read(assets_file)
                            .map_err(|e| UnityError::Other(e));
                    }
                }
            }

            // Python line 45: return None (no MeshFilter found)
            Ok(None)
        }

        _ => {
            // Not a renderer type we support
            Ok(None)
        }
    }
}

/// Export a mesh renderer to OBJ and MTL files with textures.
///
/// This function extracts the mesh, materials, and textures from a renderer
/// and saves them to the specified directory.
///
/// Python equivalent: MeshRendererExporter.py lines 48-106
pub fn export_mesh_renderer(renderer: &Renderer, export_dir: &str) -> UnityResult<()> {
    // Setup (Python lines 49-53)
    std::fs::create_dir_all(export_dir)
        .map_err(|e| UnityError::Other(format!("Failed to create directory: {}", e)))?;

    let mesh = match get_mesh(renderer)? {
        Some(m) => m,
        None => return Ok(()),
    };

    // Calculate firstSubMesh (Python lines 55-62)
    let mut first_sub_mesh: u16 = 0;
    match renderer.type_name() {
        "SkinnedMeshRenderer" => {
            // Downcast to SkinnedMeshRenderer
            if let Some(skinned) = renderer.as_any().downcast_ref::<SkinnedMeshRenderer>() {
                // Python lines 57-60: hasattr(renderer, "m_StaticBatchInfo")
                if let Some(ref batch_info) = skinned.m_StaticBatchInfo {
                    if batch_info.subMeshCount.unwrap_or(0) > 0 {
                        first_sub_mesh = batch_info.firstSubMesh.unwrap_or(0);
                    }
                }
                // Python lines 61-62: hasattr(renderer, "m_SubsetIndices")
                else if let Some(ref indices) = skinned.m_SubsetIndices {
                    // Python: min(renderer.m_SubsetIndices)
                    first_sub_mesh = *indices.iter().min().unwrap_or(&0) as u16;
                }
            }
        }
        "MeshRenderer" => {
            // Downcast to MeshRenderer
            if let Some(mesh_renderer) = renderer.as_any().downcast_ref::<MeshRenderer>() {
                // Same logic as SkinnedMeshRenderer
                if let Some(ref batch_info) = mesh_renderer.m_StaticBatchInfo {
                    if batch_info.subMeshCount.unwrap_or(0) > 0 {
                        first_sub_mesh = batch_info.firstSubMesh.unwrap_or(0);
                    }
                } else if let Some(ref indices) = mesh_renderer.m_SubsetIndices {
                    first_sub_mesh = *indices.iter().min().unwrap_or(&0) as u16;
                }
            }
        }
        _ => {} // Unknown renderer type
    }

    // Initialize vectors (Python lines 64-65)
    let mut materials = Vec::<String>::new();
    let mut material_names = Vec::<Option<String>>::new();

    // Get materials reference
    let renderer_materials: &Vec<PPtrData<Material>> = match renderer.type_name() {
        "SkinnedMeshRenderer" => {
            if let Some(skinned) = renderer.as_any().downcast_ref::<SkinnedMeshRenderer>() {
                skinned.m_Materials.as_ref().ok_or_else(|| {
                    UnityError::Other("SkinnedMeshRenderer has no m_Materials".to_string())
                })?
            } else {
                return Err(UnityError::Other(
                    "Failed to downcast to SkinnedMeshRenderer".to_string(),
                ));
            }
        }
        "MeshRenderer" => {
            if let Some(mesh_renderer) = renderer.as_any().downcast_ref::<MeshRenderer>() {
                mesh_renderer.m_Materials.as_ref().ok_or_else(|| {
                    UnityError::Other("MeshRenderer has no m_Materials".to_string())
                })?
            } else {
                return Err(UnityError::Other(
                    "Failed to downcast to MeshRenderer".to_string(),
                ));
            }
        }
        _ => return Err(UnityError::Other("Unknown renderer type".to_string())),
    };

    // Get SerializedFile
    let assets_file = renderer
        .object_reader()
        .ok_or_else(|| UnityError::Other("Renderer has no object_reader".to_string()))?
        .assets_file()
        .ok_or_else(|| UnityError::Other("No SerializedFile".to_string()))?;

    // SUBMESH LOOP (Python lines 66-88)
    for (i, _submesh) in mesh.m_SubMeshes.iter().enumerate() {
        let mat_index = i as i32 - first_sub_mesh as i32;

        if mat_index < 0 || mat_index >= renderer_materials.len() as i32 {
            continue;
        }

        let mat_ptr = &renderer_materials[mat_index as usize];

        if mat_ptr.is_null() {
            material_names.push(None);
            continue;
        }

        let mat: Material = match mat_ptr.read(assets_file) {
            Ok(m) => m,
            Err(_) => {
                material_names.push(None);
                continue;
            }
        };

        materials.push(export_material(&mat)?);
        material_names.push(mat.m_Name.clone());

        // Texture Loop (Python lines 78-88)
        if let Some(ref saved_props) = mat.m_SavedProperties {
            if let Some(ref tex_envs) = saved_props.m_TexEnvs {
                for (property_name, tex_env) in tex_envs {
                    let tex_pptr = match &tex_env.m_Texture {
                        Some(pptr) if !pptr.is_null() => pptr,
                        _ => continue,
                    };

                    let key = property_name.name.as_deref().unwrap_or("texture");

                    let tex2d_pptr =
                        PPtrData::<Texture2D>::new(tex_pptr.m_file_id, tex_pptr.m_path_id);
                    let tex: Texture2D = match tex2d_pptr.read(assets_file) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };

                    let tex_name = if !tex.m_Name.as_ref().map_or(true, |v| v.is_empty()) {
                        format!("{}.png", tex.m_Name.as_deref().unwrap_or("unnamed"))
                    } else {
                        format!("{}.png", key)
                    };

                    let output_path = format!("{}/{}", export_dir, tex_name);

                    // Python line 88: tex.read().image.save(f)
                    texture_2d_converter::save_texture_as_png(&tex, Path::new(&output_path), true)?;
                }
            }
        }
        // END TEXTURE LOOP
    }
    // END SUBMESH LOOP ← This is line 367 in your file

    // Save .obj file (Python lines 90-97) - OUTSIDE all loops
    let obj_path = format!(
        "{}/{}.obj",
        export_dir,
        mesh.m_Name.as_deref().unwrap_or("mesh")
    );

    // Convert material_names: Vec<Option<String>> → Vec<String>
    // IMPORTANT: Keep None values as empty strings to preserve index alignment with submeshes
    let material_names_vec: Vec<String> = material_names
        .iter()
        .map(|n| {
            n.as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| String::new())
        })
        .collect();

    let obj_content = export_mesh_obj(&mesh, Some(material_names_vec))?;

    std::fs::write(&obj_path, obj_content)
        .map_err(|e| UnityError::Other(format!("Failed to write OBJ: {}", e)))?;

    // Save .mtl file (Python lines 99-106) - OUTSIDE all loops
    let mtl_path = format!(
        "{}/{}.mtl",
        export_dir,
        mesh.m_Name.as_deref().unwrap_or("mesh")
    );
    let mtl_content = materials.join("\n");

    std::fs::write(&mtl_path, mtl_content)
        .map_err(|e| UnityError::Other(format!("Failed to write MTL: {}", e)))?;

    Ok(())
}

/// Creates a material file (.mtl) for the given material.
///
/// Extracts material properties (colors, floats, textures) and converts them
/// to Wavefront MTL format.
///
/// Python equivalent: MeshRendererExporter.py lines 109-182
pub fn export_material(mat: &Material) -> UnityResult<String> {
    // STEP 1: Convert property vectors to HashMaps (Python lines 117-126)
    // Python: properties_to_dict(properties)
    //   return {k if isinstance(k, str) else k.name: v for k, v in properties if v is not None}

    // Convert m_Colors: Vec<(FastPropertyName, Color)> → HashMap<String, Color>
    let colors: HashMap<String, Color> = mat
        .m_SavedProperties
        .as_ref()
        .and_then(|props| props.m_Colors.as_ref())
        .map(|colors_vec| {
            colors_vec
                .iter()
                .map(|(name, color)| (name.name.clone().unwrap_or_default(), color.clone()))
                .collect()
        })
        .unwrap_or_default();

    // Convert m_Floats: Vec<(FastPropertyName, f32)> → HashMap<String, f32>
    let floats: HashMap<String, f32> = mat
        .m_SavedProperties
        .as_ref()
        .and_then(|props| props.m_Floats.as_ref())
        .map(|floats_vec| {
            floats_vec
                .iter()
                .map(|(name, value)| (name.name.clone().unwrap_or_default(), *value))
                .collect()
        })
        .unwrap_or_default();

    // Convert m_TexEnvs: Vec<(FastPropertyName, UnityTexEnv)> → HashMap<String, UnityTexEnv>
    // (We'll use this in the loop later)
    let _tex_envs: HashMap<String, &UnityTexEnv> = mat
        .m_SavedProperties
        .as_ref()
        .and_then(|props| props.m_TexEnvs.as_ref())
        .map(|tex_envs_vec| {
            tex_envs_vec
                .iter()
                .map(|(name, tex_env)| (name.name.clone().unwrap_or_default(), tex_env))
                .collect()
        })
        .unwrap_or_default();

    // STEP 2: Extract material properties with defaults (Python lines 128-134)
    // Python: diffuse = clt(colors.get("_Color", (0.8, 0.8, 0.8, 1)))
    //   where clt() converts Color to tuple (r, g, b, a)

    // Python's clt() is just: (color.R, color.G, color.B, color.A)
    // In Rust, Color has r, g, b, a fields (lowercase)

    let diffuse = colors
        .get("_Color")
        .cloned()
        .unwrap_or(Color::new(0.8, 0.8, 0.8, 1.0));

    let ambient = colors
        .get("_SColor")
        .cloned()
        .unwrap_or(Color::new(0.2, 0.2, 0.2, 1.0));

    let specular = colors
        .get("_SpecularColor")
        .cloned()
        .unwrap_or(Color::new(0.2, 0.2, 0.2, 1.0));

    // Python line 133: shininess = floats.get("_Shininess", 20.0)
    let shininess = floats.get("_Shininess").copied().unwrap_or(20.0);

    // Python line 134: transparency = floats.get("_Transparency", 0.0)
    let transparency = floats.get("_Transparency").copied().unwrap_or(0.0);

    // STEP 3: Build MTL lines (Python lines 136-154)
    // Python: sb: List[str] = []
    let mut lines = Vec::<String>::new();

    // Python line 137: sb.append(f"newmtl {mat.m_Name}")
    lines.push(format!(
        "newmtl {}",
        mat.m_Name.as_deref().unwrap_or("unnamed")
    ));

    // Python line 140: sb.append(f"Ka {ambient[0]:.4f} {ambient[1]:.4f} {ambient[2]:.4f}")
    // Ka = ambient color (r, g, b)
    lines.push(format!(
        "Ka {:.4} {:.4} {:.4}",
        ambient.r, ambient.g, ambient.b
    ));

    // Python line 143: sb.append(f"Kd {diffuse[0]:.4f} {diffuse[1]:.4f} {diffuse[2]:.4f}")
    // Kd = diffuse color (r, g, b)
    lines.push(format!(
        "Kd {:.4} {:.4} {:.4}",
        diffuse.r, diffuse.g, diffuse.b
    ));

    // Python line 146: sb.append(f"Ks {specular[0]:.4f} {specular[1]:.4f} {specular[2]:.4f}")
    // Ks = specular color (r, g, b)
    lines.push(format!(
        "Ks {:.4} {:.4} {:.4}",
        specular.r, specular.g, specular.b
    ));

    // Python line 151: sb.append(f"Tr {transparency:.4f}")
    // Tr = transparency (0.0 = opaque, 1.0 = transparent)
    lines.push(format!("Tr {:.4}", transparency));

    // Python line 154: sb.append(f"Ns {shininess:.4f}")
    // Ns = shininess/specular exponent
    lines.push(format!("Ns {:.4}", shininess));

    // STEP 4: Process texture maps (Python lines 159-180)
    // Only process textures if there are any AND we have object_reader
    if let Some(ref saved_props) = mat.m_SavedProperties {
        if let Some(ref tex_envs) = saved_props.m_TexEnvs {
            if !tex_envs.is_empty() && mat.object_reader.is_some() {
                // Get assets_file from material's object_reader (we'll need this to read textures)
                let assets_file = mat
                    .object_reader
                    .as_ref()
                    .ok_or_else(|| UnityError::Other("Material has no object_reader".to_string()))?
                    .assets_file()
                    .ok_or_else(|| {
                        UnityError::Other("ObjectReader has no SerializedFile".to_string())
                    })?;

                // Now iterate over textures
                for (property_name, tex_env) in tex_envs {
                    // Python lines 162-163: if not texEnv.m_Texture: continue
                    let tex_pptr = match &tex_env.m_Texture {
                        Some(pptr) if !pptr.is_null() => pptr,
                        _ => continue,
                    };

                    // Python lines 164-166: if not isinstance(key, str): key = key.name
                    let key = property_name.name.as_deref().unwrap_or("texture");

                    // Python line 168: tex: Texture2D = texEnv.m_Texture.read()
                    let tex2d_pptr =
                        PPtrData::<Texture2D>::new(tex_pptr.m_file_id, tex_pptr.m_path_id);
                    let tex: Texture2D = match tex2d_pptr.read(assets_file) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };

                    // Python line 169: texName = f"{tex.m_Name if tex.m_Name else key}.png"
                    // Use tex.m_Name if not empty, otherwise use key
                    let tex_name = if !tex.m_Name.as_ref().map_or(true, |v| v.is_empty()) {
                        format!("{}.png", tex.m_Name.as_deref().unwrap_or("unnamed"))
                    } else {
                        format!("{}.png", key)
                    };

                    if key == "_MainTex" {
                        lines.push(format!("map_Kd {}", tex_name));
                    } else if key == "_BumpMap" {
                        // TODO: bump is default, some use map_bump
                        lines.push(format!("map_bump {}", tex_name));
                        lines.push(format!("bump {}", tex_name));
                    } else if key.contains("Specular") {
                        lines.push(format!("map_Ks {}", tex_name));
                    } else if key.contains("Normal") {
                        // TODO: figure out the key
                    }
                }
            }
        }
    }

    let ret = lines.join("\n");

    Ok(ret)
}
