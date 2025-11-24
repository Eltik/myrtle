//! Mesh exporter for Unity meshes
//!
//! Exports Unity Mesh objects to various 3D formats.
//! Currently supports: OBJ (Wavefront)
//!
//! Python equivalent: export/MeshExporter.py (66 lines)

use crate::classes::generated::Mesh;
use crate::errors::{UnityError, UnityResult};
use crate::helpers::mesh_helper::{MeshHandler, MeshSource};

/// Exports a mesh to the specified format
///
/// Python equivalent: export_mesh (lines 11-14)
///
/// # Arguments
/// * `mesh` - The Unity Mesh object to export
/// * `format` - Export format ("obj" currently supported)
///
/// # Returns
/// `Ok(String)` - The exported mesh data as a string
/// `Err(UnityError)` - If the format is not supported
///
/// # Example
/// ```ignore
/// let obj_data = export_mesh(&mesh, "obj")?;
/// std::fs::write("mesh.obj", obj_data)?;
/// ```
pub fn export_mesh(mesh: &Mesh, format: &str) -> UnityResult<String> {
    match format {
        "obj" => export_mesh_obj(mesh, None),
        _ => Err(UnityError::Other(format!(
            "Export format {} not implemented",
            format
        ))),
    }
}

/// Exports a mesh to Wavefront OBJ format
///
/// Python equivalent: export_mesh_obj (lines 17-66)
///
/// # Arguments
/// * `mesh` - The Unity Mesh object to export
/// * `material_names` - Optional material names for submeshes
///
/// # Returns
/// `Ok(String)` - OBJ format string
/// `Err(UnityError)` - If mesh is empty or invalid
///
/// # OBJ Format
/// - `g` - Group name
/// - `v` - Vertex position (x, y, z)
/// - `vt` - Texture coordinate (u, v)
/// - `vn` - Normal vector (x, y, z)
/// - `f` - Face (triangle indices)
///
/// # Example
/// ```ignore
/// let materials = vec!["Material1".to_string(), "Material2".to_string()];
/// let obj_data = export_mesh_obj(&mesh, Some(materials))?;
/// ```
pub fn export_mesh_obj(mesh: &Mesh, material_names: Option<Vec<String>>) -> UnityResult<String> {
    // STEP 1: Process mesh with MeshHandler (Python lines 18-19)
    // Note: Python's MeshHandler doesn't need version, but Rust does
    // We'll use a default version if not available from mesh.object_reader
    let version = (2019, 4, 0, 0); // Default version, matches Python's fallback behavior

    let mut handler = MeshHandler::new(
        MeshSource::Mesh(mesh.clone()),
        Some(version),
        None, // Default endianness (Little)
    )
    .map_err(|e| UnityError::Other(format!("Failed to create MeshHandler: {}", e)))?;

    handler
        .process()
        .map_err(|e| UnityError::Other(format!("Failed to process mesh: {}", e)))?;

    // STEP 2: Vertex count check (Python lines 22-23)
    // Python returns False (weird!), but Rust should return error
    if handler.m_vertex_count == 0 {
        return Err(UnityError::Other("Mesh has no vertices".to_string()));
    }

    // STEP 3: Initialize string builder (Python line 25)
    let mut sb = String::new();
    let name = mesh.m_Name.as_deref().unwrap_or("mesh");
    sb.push_str(&format!("g {}\n", name));

    // STEP 4: Material library (Python lines 26-27)
    if material_names.is_some() {
        sb.push_str(&format!("mtllib {}.mtl\n", name));
    }

    // STEP 5: Vertices (Python lines 29-37)
    // CRITICAL: Python checks m_Mesh.m_Vertices, returns False if None
    let vertices = handler
        .m_vertices
        .as_ref()
        .ok_or_else(|| UnityError::Other("Mesh has no vertex data".to_string()))?;

    // Write vertices with NaN handling and -x inversion
    // Python format: "v {0:.9G} {1:.9G} {2:.9G}\n".format(-pos[0], pos[1], pos[2])
    for (x, y, z) in vertices {
        let x_val = if x.is_nan() { 0.0 } else { -x };
        let y_val = if y.is_nan() { 0.0 } else { *y };
        let z_val = if z.is_nan() { 0.0 } else { *z };
        sb.push_str(&format!("v {:.9} {:.9} {:.9}\n", x_val, y_val, z_val));
    }

    // STEP 6: UVs (Python lines 41-45)
    if let Some(uvs) = &handler.m_uv0 {
        for (u, v) in uvs {
            let u_val = if u.is_nan() { 0.0 } else { *u };
            let v_val = if v.is_nan() { 0.0 } else { *v };
            sb.push_str(&format!("vt {:.9} {:.9}\n", u_val, v_val));
        }
    }

    // STEP 7: Normals (Python lines 49-53)
    if let Some(normals) = &handler.m_normals {
        for (x, y, z) in normals {
            let x_val = if x.is_nan() { 0.0 } else { -x };
            let y_val = if y.is_nan() { 0.0 } else { *y };
            let z_val = if z.is_nan() { 0.0 } else { *z };
            sb.push_str(&format!("vn {:.9} {:.9} {:.9}\n", x_val, y_val, z_val));
        }
    }

    // STEP 8: Faces/Triangles (Python lines 57-64)
    let triangles = handler
        .get_triangles()
        .map_err(|e| UnityError::Other(format!("Failed to get triangles: {}", e)))?;

    for (i, submesh_triangles) in triangles.iter().enumerate() {
        // Group for each submesh
        sb.push_str(&format!("g {}_{}\n", name, i));

        // Material if provided
        if let Some(ref materials) = material_names {
            if i < materials.len() && !materials[i].is_empty() {
                sb.push_str(&format!("usemtl {}\n", materials[i]));
            }
        }

        // Write faces (triangles)
        // Python format: "f {0}/{0}/{0} {1}/{1}/{1} {2}/{2}/{2}\n".format(c + 1, b + 1, a + 1)
        // Note: Indices are 1-based in OBJ format, and order is reversed (a,b,c -> c,b,a)
        for triangle in submesh_triangles {
            let a = triangle[0] as usize + 1;
            let b = triangle[1] as usize + 1;
            let c = triangle[2] as usize + 1;
            sb.push_str(&format!(
                "f {}/{}/{} {}/{}/{} {}/{}/{}\n",
                c, c, c, b, b, b, a, a, a
            ));
        }
    }

    Ok(sb)
}
