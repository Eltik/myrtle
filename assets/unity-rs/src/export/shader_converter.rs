//! Shader Converter - Extracts and formats Unity shader data
//!
//! This module provides functionality to export Unity Shader assets into a human-readable
//! text format. It handles three shader formats based on Unity version:
//! - Unity 5.3-5.4: Uses m_SubProgramBlob (LZ4 decompression + ShaderProgram parsing)
//! - Unity 5.5+: Uses compressedBlob (multi-platform compressed shader data)
//! - Legacy: Raw m_Script (UTF-8 decode)
//!
//! # Note
//! This is NOT a full shader decompiler. It extracts shader metadata and:
//! - Preserves source code for platforms that store it (OpenGL, Metal, Console)
//! - Shows placeholders for compiled bytecode formats (DirectX, Vulkan)
//! - Does NOT reverse-engineer DXBC or SPIRV into readable shader code
//!
//! Achieves exact parity with Python's ShaderConverter.py (738 lines)

use regex::Regex;

use crate::classes::generated::{
    SerializedPass, SerializedProperties, SerializedProperty, SerializedShader,
    SerializedShaderState, SerializedSubProgram, SerializedSubShader, SerializedTagMap, Shader,
};
use crate::errors::UnityResult;
use crate::files::object_reader::ObjectReader;
use crate::helpers::compression_helper::decompress_lz4;
use crate::streams::endian_reader::{BinaryReader, Endian, MemoryReader};
use crate::{
    PassType, SerializedPropertyType, ShaderCompilerPlatform, ShaderGpuProgramType,
    TextureDimension, UnityError,
};

// Import all enum variants for pattern matching
use PassType::*;

/// Helper function to extract version from object_reader
fn get_version(shader: &Shader) -> UnityResult<(i32, i32, i32, i32)> {
    let reader_trait = shader
        .object_reader
        .as_ref()
        .ok_or_else(|| UnityError::Other("Shader has no object_reader".to_string()))?;

    // Downcast to concrete ObjectReader<Shader> type
    let reader = reader_trait
        .as_any()
        .downcast_ref::<ObjectReader<Shader>>()
        .ok_or_else(|| UnityError::Other("Failed to downcast object_reader".to_string()))?;

    // Convert u32 version to i32 for compatibility
    let (v0, v1, v2, v3) = reader.version;
    Ok((v0 as i32, v1 as i32, v2 as i32, v3 as i32))
}

/// Header text added to all exported shaders
const HEADER: &str = r#"//////////////////////////////////////////
//
// NOTE: This is *not* a valid shader file
//
///////////////////////////////////////////
"#;

/// Main entry point for shader export
///
/// Python: lines 40-54
pub fn export_shader(shader: &Shader) -> UnityResult<String> {
    // Unity 5.3 - 5.4: m_SubProgramBlob format
    if let Some(ref sub_program_blob) = shader.m_SubProgramBlob {
        if !sub_program_blob.is_empty() {
            let decompressed_size = shader.decompressedSize.unwrap_or(0) as usize;
            let decompressed = decompress_lz4(sub_program_blob, decompressed_size)?;

            let version = get_version(shader)?;

            let mut reader = MemoryReader::new(decompressed, Endian::Little, 0);
            let program = ShaderProgram::new(&mut reader, version)?;

            let script = shader.m_Script.as_deref().unwrap_or("");
            return Ok(format!("{}{}", HEADER, program.export(script)?));
        }
    }

    // Unity 5.5+: compressedBlob format
    if let Some(ref compressed_blob) = shader.compressedBlob {
        if !compressed_blob.is_empty() {
            return Ok(format!("{}{}", HEADER, convert_serialized_shader(shader)?));
        }
    }

    // Legacy: raw m_Script
    let script = shader.m_Script.as_deref().unwrap_or("");
    Ok(format!("{}{}", HEADER, script))
}

/// Convert Unity 5.5+ serialized shader format
///
/// Python: lines 57-94
fn convert_serialized_shader(shader: &Shader) -> UnityResult<String> {
    let mut shader_programs = Vec::new();

    let platforms = shader
        .platforms
        .as_ref()
        .ok_or_else(|| UnityError::Other("No platforms".to_string()))?;
    let platform_count = platforms.len();

    let compressed_blob = shader
        .compressedBlob
        .as_ref()
        .ok_or_else(|| UnityError::Other("No compressed blob".to_string()))?;

    let compressed_lengths = shader
        .compressedLengths
        .as_ref()
        .ok_or_else(|| UnityError::Other("No compressedLengths".to_string()))?;
    let decompressed_lengths = shader
        .decompressedLengths
        .as_ref()
        .ok_or_else(|| UnityError::Other("No decompressedLengths".to_string()))?;
    let offsets = shader
        .offsets
        .as_ref()
        .ok_or_else(|| UnityError::Other("No offsets".to_string()))?;

    // Helper to get entry from nested Vec (Python lines 63-67)
    fn get_entry_u32(array: &[Vec<u32>], index: usize) -> Option<u32> {
        array.get(index)?.first().copied()
    }

    fn get_entry_u32_2d(array: &[Vec<u32>], index: usize, sub_index: usize) -> Option<u32> {
        array.get(index)?.get(sub_index).copied()
    }

    let version = get_version(shader)?;

    for i in 0..platform_count {
        // Python lines 70-74: bounds check
        if i >= compressed_lengths.len() || i >= decompressed_lengths.len() {
            break;
        }

        let compressed_size = get_entry_u32(compressed_lengths, i)
            .ok_or_else(|| UnityError::Other(format!("Missing compressedLength at {}", i)))?;
        let decompressed_size = get_entry_u32(decompressed_lengths, i)
            .ok_or_else(|| UnityError::Other(format!("Missing decompressedLength at {}", i)))?;
        let offset = get_entry_u32(offsets, i)
            .ok_or_else(|| UnityError::Other(format!("Missing offset at {}", i)))?
            as i32;

        // Python lines 80-83: extract and decompress
        let start = offset as usize;
        let end = start + compressed_size as usize;

        if end > compressed_blob.len() {
            return Err(UnityError::Other(format!(
                "Compressed blob offset out of bounds: {} > {}",
                end,
                compressed_blob.len()
            )));
        }

        let compressed_bytes = &compressed_blob[start..end];
        let decompressed = decompress_lz4(compressed_bytes, decompressed_size as usize)?;

        // Python lines 85-90: create ShaderProgram
        let mut reader = MemoryReader::new(decompressed, Endian::Little, 0);
        shader_programs.push(ShaderProgram::new(&mut reader, version)?);
    }

    // Python lines 92-94: convert parsed form
    let parsed_form = shader
        .m_ParsedForm
        .as_ref()
        .ok_or_else(|| UnityError::Other("No parsed form".to_string()))?;

    let platforms_i32: Vec<i32> = shader
        .platforms
        .as_ref()
        .ok_or_else(|| UnityError::Other("No platforms".to_string()))?
        .iter()
        .map(|&p| p as i32)
        .collect();

    convert_serialized_shader_parsed_form(parsed_form, &platforms_i32, &shader_programs)
}

/// Convert serialized shader to text format
///
/// Python: lines 97-119
fn convert_serialized_shader_parsed_form(
    parsed_form: &SerializedShader,
    platforms: &[i32],
    shader_programs: &[ShaderProgram],
) -> UnityResult<String> {
    let mut sb = String::new();

    // Python lines 100-103: shader name
    sb.push_str("Shader \"");
    sb.push_str(parsed_form.m_Name.as_deref().unwrap_or("Shader"));
    sb.push_str("\" {\n");

    // Python line 104: properties
    sb.push_str(&convert_serialized_properties(
        parsed_form
            .m_PropInfo
            .as_ref()
            .unwrap_or(&SerializedProperties::default()),
    )?);

    // Python lines 105-109: subshaders
    if let Some(ref sub_shaders) = parsed_form.m_SubShaders {
        for sub_shader in sub_shaders {
            sb.push_str(&convert_serialized_sub_shader(
                sub_shader,
                platforms,
                shader_programs,
            )?);
        }
    }

    // Python lines 111-114: fallback
    if !parsed_form
        .m_FallbackName
        .as_ref()
        .map_or(true, |v| v.is_empty())
    {
        sb.push_str(&format!(
            "Fallback \"{}\"\n",
            parsed_form.m_FallbackName.as_deref().unwrap_or("")
        ));
    }

    // Python lines 115-117: custom editor
    if !parsed_form
        .m_CustomEditorName
        .as_ref()
        .map_or(true, |v| v.is_empty())
    {
        sb.push_str(&format!(
            "CustomEditor \"{}\"\n",
            parsed_form.m_CustomEditorName.as_deref().unwrap_or("")
        ));
    }

    sb.push_str("}\n");
    Ok(sb)
}

/// Convert SubShader
///
/// Python: lines 122-139
fn convert_serialized_sub_shader(
    sub_shader: &SerializedSubShader,
    platforms: &[i32],
    shader_programs: &[ShaderProgram],
) -> UnityResult<String> {
    let mut sb = String::from("SubShader {\n");

    // Python lines 129-130: LOD
    if sub_shader.m_LOD.unwrap_or(0) != 0 {
        sb.push_str(&format!(" LOD {}\n", sub_shader.m_LOD.unwrap_or(0)));
    }

    // Python line 132: tags
    if let Some(ref tags) = sub_shader.m_Tags {
        sb.push_str(&convert_serialized_tag_map(tags, 1)?);
    }

    // Python lines 134-136: passes
    if let Some(ref passes) = sub_shader.m_Passes {
        for pass in passes {
            sb.push_str(&convert_serialized_pass(pass, platforms, shader_programs)?);
        }
    }

    sb.push_str("}\n");
    Ok(sb)
}

/// Convert Pass
///
/// Python: lines 142-212
fn convert_serialized_pass(
    pass: &SerializedPass,
    platforms: &[i32],
    shader_programs: &[ShaderProgram],
) -> UnityResult<String> {
    let mut sb = String::new();

    // Python lines 146-151: pass type
    let pass_type = PassType::from(pass.m_Type.unwrap_or(0) as u32);
    match pass_type {
        kPassTypeNormal => sb.push_str(" Pass "),
        kPassTypeUse => sb.push_str(" UsePass "),
        kPassTypeGrab => sb.push_str(" GrabPass "),
        PassType::Unknown(_) => sb.push_str(" Pass "), // Default to Pass for unknown types
    }

    // Python lines 153-154: UsePass name
    if pass_type == kPassTypeUse {
        sb.push_str(&format!(
            "\"{}\"\n",
            pass.m_UseName.as_deref().unwrap_or("")
        ));
    } else {
        sb.push_str("{\n");

        // Python lines 158-160: GrabPass texture name
        if pass_type == kPassTypeGrab {
            if !pass.m_TextureName.as_ref().map_or(true, |v| v.is_empty()) {
                sb.push_str(&format!(
                    " \"{}\"\n",
                    pass.m_TextureName.as_deref().unwrap_or("")
                ));
            }
        } else {
            // Python line 163: state
            if let Some(ref state) = pass.m_State {
                sb.push_str(&convert_serialized_shader_state(state)?);
            }

            // Python lines 165-172: vertex program
            if let Some(ref prog) = pass.progVertex {
                if let Some(ref sub_programs) = prog.m_SubPrograms {
                    if !sub_programs.is_empty() {
                        sb.push_str("Program \"vp\" {\n");
                        sb.push_str(&convert_serialized_sub_programs(
                            sub_programs,
                            platforms,
                            shader_programs,
                        )?);
                        sb.push_str("}\n");
                    }
                }
            }

            // Python lines 174-181: fragment program
            if let Some(ref prog) = pass.progFragment {
                if let Some(ref sub_programs) = prog.m_SubPrograms {
                    if !sub_programs.is_empty() {
                        sb.push_str("Program \"fp\" {\n");
                        sb.push_str(&convert_serialized_sub_programs(
                            sub_programs,
                            platforms,
                            shader_programs,
                        )?);
                        sb.push_str("}\n");
                    }
                }
            }

            // Python lines 183-190: geometry program
            if let Some(ref prog) = pass.progGeometry {
                if let Some(ref sub_programs) = prog.m_SubPrograms {
                    if !sub_programs.is_empty() {
                        sb.push_str("Program \"gp\" {\n");
                        sb.push_str(&convert_serialized_sub_programs(
                            sub_programs,
                            platforms,
                            shader_programs,
                        )?);
                        sb.push_str("}\n");
                    }
                }
            }

            // Python lines 192-199: hull program
            if let Some(ref prog) = pass.progHull {
                if let Some(ref sub_programs) = prog.m_SubPrograms {
                    if !sub_programs.is_empty() {
                        sb.push_str("Program \"hp\" {\n");
                        sb.push_str(&convert_serialized_sub_programs(
                            sub_programs,
                            platforms,
                            shader_programs,
                        )?);
                        sb.push_str("}\n");
                    }
                }
            }

            // Python lines 201-208: domain program
            if let Some(ref prog) = pass.progDomain {
                if let Some(ref sub_programs) = prog.m_SubPrograms {
                    if !sub_programs.is_empty() {
                        sb.push_str("Program \"dp\" {\n");
                        sb.push_str(&convert_serialized_sub_programs(
                            sub_programs,
                            platforms,
                            shader_programs,
                        )?);
                        sb.push_str("}\n");
                    }
                }
            }
        }

        sb.push_str("}\n");
    }

    Ok(sb)
}

/// Convert SubPrograms with grouping
///
/// Python: lines 215-266
fn convert_serialized_sub_programs(
    sub_programs: &[SerializedSubProgram],
    platforms: &[i32],
    shader_programs: &[ShaderProgram],
) -> UnityResult<String> {
    let mut sb = String::new();

    // Python lines 222-228: group by blob index, then by GPU type
    // Manual groupby implementation since we don't have itertools
    let mut sorted_by_blob: Vec<_> = sub_programs.iter().collect();
    sorted_by_blob.sort_by_key(|sp| sp.m_BlobIndex);

    let mut i = 0;
    while i < sorted_by_blob.len() {
        let blob_index = sorted_by_blob[i].m_BlobIndex.unwrap_or(0);
        let mut blob_group = Vec::new();

        // Collect all with same blob_index
        while i < sorted_by_blob.len() && sorted_by_blob[i].m_BlobIndex.unwrap_or(0) == blob_index {
            blob_group.push(sorted_by_blob[i]);
            i += 1;
        }

        // Python lines 232-234: group by GPU program type
        blob_group.sort_by_key(|sp| sp.m_GpuProgramType.unwrap_or(0) as i32);

        let mut j = 0;
        while j < blob_group.len() {
            let program_type = blob_group[j].m_GpuProgramType.unwrap_or(0);
            let mut type_group = Vec::new();

            // Collect all with same program type
            while j < blob_group.len()
                && blob_group[j].m_GpuProgramType.unwrap_or(0) == program_type
            {
                type_group.push(blob_group[j]);
                j += 1;
            }

            // Python line 236: is_tier check
            let is_tier = type_group.len() > 1;

            // Python lines 237-264: iterate platforms
            for (platform_idx, &platform) in platforms.iter().enumerate() {
                if platform_idx >= shader_programs.len() {
                    break;
                }

                // Python line 244: check if GPU program usable
                if check_gpu_program_usable(platform, program_type as i32)? {
                    for sub_program in &type_group {
                        // Python lines 246-248: SubProgram header
                        sb.push_str(&format!("SubProgram \"{} ", get_platform_string(platform)?));

                        // Python lines 250-253: hardware tier
                        if is_tier {
                            sb.push_str(&format!(
                                "hw_tier{:02} ",
                                sub_program.m_ShaderHardwareTier.unwrap_or(0)
                            ));
                        }

                        sb.push_str("\" {\n");

                        // Python lines 256-260: export subprogram
                        let blob_index = sub_program.m_BlobIndex.unwrap_or(0) as usize;
                        if blob_index < shader_programs[platform_idx].subprograms.len() {
                            sb.push_str(
                                &shader_programs[platform_idx].subprograms[blob_index].export()?,
                            );
                        }

                        sb.push_str("\n}\n");
                    }

                    break;
                }
            }
        }
    }

    Ok(sb)
}

/// Convert shader state (render settings)
///
/// Python: lines 269-335
fn convert_serialized_shader_state(state: &SerializedShaderState) -> UnityResult<String> {
    let mut sb = String::new();

    // Python lines 271-272: Name
    if !state.m_Name.as_ref().map_or(true, |v| v.is_empty()) {
        sb.push_str(&format!(
            " Name \"{}\"\n",
            state.m_Name.as_deref().unwrap_or("")
        ));
    }

    // Python lines 274-275: LOD
    if state.m_LOD.unwrap_or(0) != 0 {
        sb.push_str(&format!("  LOD {}\n", state.m_LOD.unwrap_or(0)));
    }

    // Python line 277: Tags
    if let Some(ref tags) = state.m_Tags {
        sb.push_str(&convert_serialized_tag_map(tags, 2)?);
    }

    // Python line 279: RTBlendState (commented out)
    // sb.push_str(&convert_rt_blend_state(&state.rtBlend)?);

    // Python lines 281-282: AlphaToMask
    if let Some(ref alpha_to_mask) = state.alphaToMask {
        if alpha_to_mask.val.unwrap_or(0.0) > 0.0 {
            sb.push_str(" AlphaToMask On\n");
        }
    }

    // Python lines 284-285: ZClip
    if let Some(ref zclip) = state.zClip {
        if zclip.val.unwrap_or(1.0) != 1.0 {
            sb.push_str(" ZClip Off\n");
        }
    }

    // Python lines 287-306: ZTest
    if let Some(ref ztest) = state.zTest {
        let ztest_val = ztest.val.unwrap_or(4.0);
        if ztest_val != 4.0 {
            sb.push_str(" ZTest ");
            match ztest_val as i32 {
                0 => sb.push_str("Off"),      // kFuncDisabled
                1 => sb.push_str("Never"),    // kFuncNever
                2 => sb.push_str("Less"),     // kFuncLess
                3 => sb.push_str("Equal"),    // kFuncEqual
                5 => sb.push_str("Greater"),  // kFuncGreater
                6 => sb.push_str("NotEqual"), // kFuncNotEqual
                7 => sb.push_str("GEqual"),   // kFuncGEqual
                8 => sb.push_str("Always"),   // kFuncAlways
                _ => {}
            }
            sb.push_str("\n");
        }
    }

    // Python lines 308-309: ZWrite
    if let Some(ref zwrite) = state.zWrite {
        if zwrite.val.unwrap_or(1.0) != 1.0 {
            sb.push_str(" ZWrite Off\n");
        }
    }

    // Python lines 311-318: Culling
    if let Some(ref culling) = state.culling {
        let culling_val = culling.val.unwrap_or(2.0);
        if culling_val != 2.0 {
            sb.push_str(" Cull ");
            match culling_val as i32 {
                0 => sb.push_str("Off"),   // kCullOff
                1 => sb.push_str("Front"), // kCullFront
                _ => {}
            }
            sb.push_str("\n");
        }
    }

    // Python lines 320-325: Offset
    let offset_factor_val = state
        .offsetFactor
        .as_ref()
        .and_then(|v| v.val)
        .unwrap_or(0.0);
    let offset_units_val = state
        .offsetUnits
        .as_ref()
        .and_then(|v| v.val)
        .unwrap_or(0.0);
    if offset_factor_val != 0.0 || offset_units_val != 0.0 {
        sb.push_str(&format!(
            " Offset {}, {}\n",
            offset_factor_val, offset_units_val
        ));
    }

    // Python line 327: TODO Stencil (commented out in Python)

    // Python line 329: TODO Fog (commented out in Python)

    // Python lines 331-332: Lighting
    let lighting_str = if state.lighting.unwrap_or(false) {
        "On"
    } else {
        "Off"
    };
    sb.push_str(&format!("  Lighting {}\n", lighting_str));

    // Python line 334: GpuProgramID
    sb.push_str(&format!(
        "  GpuProgramID {}\n",
        state.gpuProgramID.unwrap_or(0)
    ));

    Ok(sb)
}

/// Stub for RTBlendState conversion (empty in Python too)
///
/// Python: lines 338-341
#[allow(dead_code)]
fn convert_rt_blend_state(_rb_blend: &SerializedShaderState) -> UnityResult<String> {
    // TODO Blend (Python has this as TODO too)
    Ok(String::new())
}

/// Convert tag map
///
/// Python: lines 344-352
fn convert_serialized_tag_map(tags: &SerializedTagMap, indent: usize) -> UnityResult<String> {
    let tags_vec = match &tags.tags {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(String::new()),
    };

    let mut sb = String::new();
    sb.push_str(&" ".repeat(indent));
    sb.push_str("Tags { ");

    for (key, value) in tags_vec {
        sb.push_str(&format!("\"{}\" = \"{}\" ", key, value));
    }

    sb.push_str("}\n");
    Ok(sb)
}

/// Convert properties block
///
/// Python: lines 355-360
fn convert_serialized_properties(props: &SerializedProperties) -> UnityResult<String> {
    let mut sb = String::from("Properties {\n");

    if let Some(ref props_vec) = props.m_Props {
        for prop in props_vec {
            sb.push_str(&convert_serialized_property(prop)?);
        }
    }

    sb.push_str("}\n");
    Ok(sb)
}

/// Convert individual property
///
/// Python: lines 363-413
fn convert_serialized_property(prop: &SerializedProperty) -> UnityResult<String> {
    let mut sb = String::new();

    // Python line 364: attributes
    if let Some(ref attrs) = prop.m_Attributes {
        for attr in attrs {
            sb.push_str(&format!("[{}] ", attr));
        }
    }

    // Python line 365: name and description
    sb.push_str(&format!(
        "{} (\"{}\", ",
        prop.m_Name.as_deref().unwrap_or(""),
        prop.m_Description.as_deref().unwrap_or("")
    ));

    // Python lines 367-389: property type
    let prop_type = SerializedPropertyType::from(prop.m_Type.unwrap_or(0) as u32);
    match prop_type {
        SerializedPropertyType::kColor => sb.push_str("Color"),
        SerializedPropertyType::kVector => sb.push_str("Vector"),
        SerializedPropertyType::kFloat => sb.push_str("Float"),
        SerializedPropertyType::kRange => {
            sb.push_str(&format!(
                "Range({}, {})",
                prop.m_DefValue_1_.unwrap_or(0.0),
                prop.m_DefValue_2_.unwrap_or(0.0)
            ));
        }
        SerializedPropertyType::kTexture => {
            let tex_dim = prop
                .m_DefTexture
                .as_ref()
                .and_then(|t| t.m_TexDim)
                .and_then(|d| TextureDimension::from_i32(d))
                .unwrap_or(TextureDimension::kTexDimAny);
            match tex_dim {
                TextureDimension::kTexDimAny => sb.push_str("any"),
                TextureDimension::kTexDim2D => sb.push_str("2D"),
                TextureDimension::kTexDim3D => sb.push_str("3D"),
                TextureDimension::kTexDimCUBE => sb.push_str("Cube"),
                TextureDimension::kTexDim2DArray => sb.push_str("2DArray"),
                TextureDimension::kTexDimCubeArray => sb.push_str("CubeArray"),
                _ => {}
            }
        }
        SerializedPropertyType::Unknown(_) => {
            return Err(UnityError::Other(format!(
                "Unknown property type: {}",
                prop.m_Type.unwrap_or(0)
            )));
        }
    }

    sb.push_str(") = ");

    // Python lines 393-410: default value
    match prop_type {
        SerializedPropertyType::kColor | SerializedPropertyType::kVector => {
            sb.push_str(&format!(
                "({},{},{},{})",
                prop.m_DefValue_0_.unwrap_or(0.0),
                prop.m_DefValue_1_.unwrap_or(0.0),
                prop.m_DefValue_2_.unwrap_or(0.0),
                prop.m_DefValue_3_.unwrap_or(0.0)
            ));
        }
        SerializedPropertyType::kFloat | SerializedPropertyType::kRange => {
            sb.push_str(&format!("{}", prop.m_DefValue_0_.unwrap_or(0.0)));
        }
        SerializedPropertyType::kTexture => {
            let default_name = prop
                .m_DefTexture
                .as_ref()
                .and_then(|t| t.m_DefaultName.as_deref())
                .unwrap_or("");
            sb.push_str(&format!("\"{}\" {{ }}", default_name));
        }
        SerializedPropertyType::Unknown(_) => {
            return Err(UnityError::Other(format!(
                "Unknown property type: {}",
                prop.m_Type.unwrap_or(0)
            )));
        }
    }

    sb.push_str("\n");
    Ok(sb)
}

/// Check if GPU program is usable on platform
///
/// Python: lines 416-534
fn check_gpu_program_usable(platform: i32, program_type: i32) -> UnityResult<bool> {
    use ShaderCompilerPlatform::*;
    use ShaderGpuProgramType::*;

    let platform_enum = ShaderCompilerPlatform::from(platform);
    let program_enum = ShaderGpuProgramType::from(program_type as u32);

    match platform_enum {
        kShaderCompPlatformGL => Ok(program_enum == kShaderGpuProgramGLLegacy),

        kShaderCompPlatformD3D9 => Ok(matches!(
            program_enum,
            kShaderGpuProgramDX9VertexSM20
                | kShaderGpuProgramDX9VertexSM30
                | kShaderGpuProgramDX9PixelSM20
                | kShaderGpuProgramDX9PixelSM30
        )),

        kShaderCompPlatformXbox360 => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        kShaderCompPlatformD3D11 => Ok(matches!(
            program_enum,
            kShaderGpuProgramDX11VertexSM40
                | kShaderGpuProgramDX11VertexSM50
                | kShaderGpuProgramDX11PixelSM40
                | kShaderGpuProgramDX11PixelSM50
                | kShaderGpuProgramDX11GeometrySM40
                | kShaderGpuProgramDX11GeometrySM50
                | kShaderGpuProgramDX11HullSM50
                | kShaderGpuProgramDX11DomainSM50
        )),

        kShaderCompPlatformGLES20 => Ok(program_enum == kShaderGpuProgramGLES),

        kShaderCompPlatformNaCl => Err(UnityError::Other(
            "NaCl platform not implemented".to_string(),
        )),

        kShaderCompPlatformFlash => Err(UnityError::Other(
            "Flash platform not implemented".to_string(),
        )),

        kShaderCompPlatformD3D11_9x => Ok(matches!(
            program_enum,
            kShaderGpuProgramDX10Level9Vertex | kShaderGpuProgramDX10Level9Pixel
        )),

        kShaderCompPlatformGLES3Plus => Ok(matches!(
            program_enum,
            kShaderGpuProgramGLES31AEP | kShaderGpuProgramGLES31 | kShaderGpuProgramGLES3
        )),

        kShaderCompPlatformPSP2 => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        kShaderCompPlatformPS4 => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        kShaderCompPlatformXboxOne => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        kShaderCompPlatformPSM => Err(UnityError::Other(
            "PSM platform not implemented".to_string(),
        )),

        kShaderCompPlatformMetal => Ok(matches!(
            program_enum,
            kShaderGpuProgramMetalVS | kShaderGpuProgramMetalFS
        )),

        kShaderCompPlatformOpenGLCore => Ok(matches!(
            program_enum,
            kShaderGpuProgramGLCore32 | kShaderGpuProgramGLCore41 | kShaderGpuProgramGLCore43
        )),

        kShaderCompPlatformN3DS => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        kShaderCompPlatformWiiU => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        kShaderCompPlatformVulkan => Ok(program_enum == kShaderGpuProgramSPIRV),

        kShaderCompPlatformSwitch => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        kShaderCompPlatformXboxOneD3D12 => Ok(matches!(
            program_enum,
            kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS
        )),

        _ => Err(UnityError::Other(format!("Unknown platform: {}", platform))),
    }
}

/// Get platform name string
///
/// Python: lines 537-581
fn get_platform_string(platform: i32) -> UnityResult<String> {
    use ShaderCompilerPlatform::*;

    let platform_enum = ShaderCompilerPlatform::from(platform);

    Ok(match platform_enum {
        kShaderCompPlatformGL => "openGL",
        kShaderCompPlatformD3D9 => "d3d9",
        kShaderCompPlatformXbox360 => "xbox360",
        kShaderCompPlatformPS3 => "ps3",
        kShaderCompPlatformD3D11 => "d3d11",
        kShaderCompPlatformGLES20 => "gles",
        kShaderCompPlatformNaCl => "glesdesktop",
        kShaderCompPlatformFlash => "flash",
        kShaderCompPlatformD3D11_9x => "d3d11_9x",
        kShaderCompPlatformGLES3Plus => "gles3",
        kShaderCompPlatformPSP2 => "psp2",
        kShaderCompPlatformPS4 => "ps4",
        kShaderCompPlatformXboxOne => "xboxone",
        kShaderCompPlatformPSM => "psm",
        kShaderCompPlatformMetal => "metal",
        kShaderCompPlatformOpenGLCore => "glcore",
        kShaderCompPlatformN3DS => "n3ds",
        kShaderCompPlatformWiiU => "wiiu",
        kShaderCompPlatformVulkan => "vulkan",
        kShaderCompPlatformSwitch => "switch",
        kShaderCompPlatformXboxOneD3D12 => "xboxone+_d3d12",
        _ => "unknown",
    }
    .to_string())
}

/// ShaderProgram struct - parses shader binary blob
///
/// Python: lines 584-609
struct ShaderProgram {
    subprograms: Vec<ShaderSubProgram>,
}

impl ShaderProgram {
    /// Parse shader program from binary reader
    ///
    /// Python: lines 587-600
    fn new<R: BinaryReader>(reader: &mut R, version: (i32, i32, i32, i32)) -> UnityResult<Self> {
        let sub_program_capacity = reader.read_i32()? as usize;
        let mut subprograms = Vec::with_capacity(sub_program_capacity);

        // Python lines 591-594: version-dependent entry size
        let entry_size = if version >= (2019, 3, 0, 0) {
            12 // Unity 2019.3 and up
        } else {
            8 // Older versions
        };

        // Python lines 596-600: read offset table and create subprograms
        for i in 0..sub_program_capacity {
            reader.set_position(4 + i * entry_size);
            let offset = reader.read_i32()? as usize;
            reader.set_position(offset);
            subprograms.push(ShaderSubProgram::new(reader)?);
        }

        Ok(ShaderProgram { subprograms })
    }

    /// Export with GpuProgramIndex substitution
    ///
    /// Python: lines 602-609
    fn export(&self, shader: &str) -> UnityResult<String> {
        let re = Regex::new(r"GpuProgramIndex (.+)").unwrap();

        let result = re.replace_all(shader, |caps: &regex::Captures| {
            if let Ok(index) = caps[1].parse::<usize>() {
                if index < self.subprograms.len() {
                    return self.subprograms[index].export().unwrap_or_default();
                }
            }
            String::new()
        });

        Ok(result.to_string())
    }
}

/// ShaderSubProgram struct - individual GPU program
///
/// Python: lines 612-737
struct ShaderSubProgram {
    version: i32,
    program_type: ShaderGpuProgramType,
    keywords: Vec<String>,
    local_keywords: Option<Vec<String>>,
    program_code: Vec<u8>,
}

impl ShaderSubProgram {
    /// Parse GPU program from binary reader
    ///
    /// Python: lines 619-649
    fn new<R: BinaryReader>(reader: &mut R) -> UnityResult<Self> {
        // Python lines 620-628: version comments
        let version = reader.read_i32()?;
        let program_type = ShaderGpuProgramType::from(reader.read_i32()? as u32);

        // Python line 632: skip 12 bytes
        reader.set_position(reader.position() + 12);

        // Python lines 634-635: version check
        if version >= 201608170 {
            reader.set_position(reader.position() + 4);
        }

        // Python lines 637-638: keywords
        let keyword_size = reader.read_i32()? as usize;
        let mut keywords = Vec::with_capacity(keyword_size);
        for _ in 0..keyword_size {
            keywords.push(reader.read_aligned_string()?);
        }

        // Python lines 640-646: local keywords (version-dependent)
        let local_keywords = if version >= 201806140 && version < 202012090 {
            let local_keyword_size = reader.read_i32()? as usize;
            let mut local_kw = Vec::with_capacity(local_keyword_size);
            for _ in 0..local_keyword_size {
                local_kw.push(reader.read_aligned_string()?);
            }
            Some(local_kw)
        } else {
            None
        };

        // Python lines 648-649: program code
        let program_code = reader.read_byte_array()?;
        reader.align_stream(4);

        Ok(ShaderSubProgram {
            version,
            program_type,
            keywords,
            local_keywords,
            program_code,
        })
    }

    /// Export shader subprogram to text
    ///
    /// Python: lines 651-737
    fn export(&self) -> UnityResult<String> {
        let mut sb = String::new();

        // Python lines 654-659: keywords
        if !self.keywords.is_empty() {
            sb.push_str("Keywords { ");
            for keyword in &self.keywords {
                sb.push_str(&format!("\"{}\" ", keyword));
            }
            sb.push_str("}\n");
        }

        // Python lines 661-669: local keywords
        if let Some(ref local_kw) = self.local_keywords {
            if !local_kw.is_empty() {
                sb.push_str("Local Keywords { ");
                for keyword in local_kw {
                    sb.push_str(&format!("\"{}\" ", keyword));
                }
                sb.push_str("}\n");
            }
        }

        sb.push_str("\"");

        // Python lines 673-734: program code based on type
        if !self.program_code.is_empty() {
            use ShaderGpuProgramType::*;

            match self.program_type {
                // Python lines 674-684: OpenGL variants (UTF-8 decode)
                kShaderGpuProgramGLLegacy
                | kShaderGpuProgramGLES31AEP
                | kShaderGpuProgramGLES31
                | kShaderGpuProgramGLES3
                | kShaderGpuProgramGLES
                | kShaderGpuProgramGLCore32
                | kShaderGpuProgramGLCore41
                | kShaderGpuProgramGLCore43 => {
                    sb.push_str(&String::from_utf8_lossy(&self.program_code));
                }

                // Python lines 685-691: DirectX 9 (placeholder)
                kShaderGpuProgramDX9VertexSM20
                | kShaderGpuProgramDX9VertexSM30
                | kShaderGpuProgramDX9PixelSM20
                | kShaderGpuProgramDX9PixelSM30 => {
                    sb.push_str("// shader disassembly not supported on DXBC");
                }

                // Python lines 692-704: DirectX 10/11 (placeholder)
                kShaderGpuProgramDX10Level9Vertex
                | kShaderGpuProgramDX10Level9Pixel
                | kShaderGpuProgramDX11VertexSM40
                | kShaderGpuProgramDX11VertexSM50
                | kShaderGpuProgramDX11PixelSM40
                | kShaderGpuProgramDX11PixelSM50
                | kShaderGpuProgramDX11GeometrySM40
                | kShaderGpuProgramDX11GeometrySM50
                | kShaderGpuProgramDX11HullSM50
                | kShaderGpuProgramDX11DomainSM50 => {
                    sb.push_str("// shader disassembly not supported on DXBC");
                }

                // Python lines 705-717: Metal (special 0xF00DCAFE header)
                kShaderGpuProgramMetalVS | kShaderGpuProgramMetalFS => {
                    let mut cursor =
                        MemoryReader::new(self.program_code.clone(), Endian::Little, 0);
                    let four_cc = cursor.read_u32()?;

                    if four_cc == 0xF00DCAFE {
                        let offset = cursor.read_i32()? as usize;
                        cursor.set_position(offset);

                        // Read null-terminated string
                        let _entry_name = cursor.read_string_to_null(1024)?;

                        // Read remaining bytes
                        let remaining = cursor.len() - cursor.position();
                        let buff = cursor.read_bytes(remaining)?;
                        sb.push_str(&String::from_utf8_lossy(&buff));
                    }
                }

                // Python lines 718-720: SPIRV (TODO - not implemented in Python)
                kShaderGpuProgramSPIRV => {
                    // TODO SpirVShaderConverter (Python has this as TODO too)
                }

                // Python lines 721-728: Console (UTF-8 decode)
                kShaderGpuProgramConsoleVS
                | kShaderGpuProgramConsoleFS
                | kShaderGpuProgramConsoleHS
                | kShaderGpuProgramConsoleDS
                | kShaderGpuProgramConsoleGS => {
                    sb.push_str(&String::from_utf8_lossy(&self.program_code));
                }

                // Python lines 729-734: Other (placeholder)
                _ => {
                    sb.push_str(&format!(
                        "//shader disassembly not supported on {:?}",
                        self.program_type
                    ));
                }
            }
        }

        sb.push_str("\"");
        Ok(sb)
    }
}
