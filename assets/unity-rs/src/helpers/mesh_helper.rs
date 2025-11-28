//! MeshHelper - Unity mesh data processor
//!
//! Processes Unity mesh vertex data, handles decompression, and extracts:
//! - Vertices (3D positions)
//! - Normals (lighting vectors)
//! - UVs (texture coordinates, channels 0-7)
//! - Colors (vertex colors)
//! - Tangents (normal mapping vectors)
//! - Bone weights/indices (for skinned meshes)
//! - Index buffers (triangles)
//!
//! Python equivalent: helpers/MeshHelper.py (734 lines)

use crate::classes::generated::*;
use crate::enums::mesh_topology::MeshTopology;
use crate::helpers::packed_bit_vector::{unpack_floats, unpack_ints, ReshapedData};
use crate::math::{Vector2, Vector3, Vector4};
use crate::streams::endian::Endian;
use std::io;

// Type aliases matching Python (lines 32-34)
type Tuple2f = (f32, f32);
type Tuple3f = (f32, f32, f32);
type Tuple4f = (f32, f32, f32, f32);
type Tuple4i = (i32, i32, i32, i32);

// =============================================================================
// SECTION 1: HELPER FUNCTIONS
// =============================================================================

/// Converts flat list to tuples
///
/// Python equivalent: MeshHelper.py lines 39-40
///
/// # Arguments
/// * `data` - Flat array of values
/// * `item_size` - Number of items per tuple (e.g., 3 for Vec3)
///
/// # Returns
/// Vector of tuples (as vectors)
///
/// # Example
/// ```ignore
/// let flat = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
/// let tuples = flat_list_to_tuples(&flat, 3);
/// // Returns: vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]
/// ```
pub fn flat_list_to_tuples<T: Clone>(data: &[T], item_size: usize) -> Vec<Vec<T>> {
    data.chunks(item_size).map(|chunk| chunk.to_vec()).collect()
}

/// Converts Vector2 list to tuples
///
/// Python equivalent: MeshHelper.py lines 43-57
///
/// # Arguments
/// * `data` - Slice of Vector2 structs
///
/// # Returns
/// Vector of (f32, f32) tuples
pub fn vector2_to_tuples(data: &[Vector2]) -> Vec<Tuple2f> {
    data.iter().map(|v| (v.x, v.y)).collect()
}

/// Converts Vector3 list to tuples
///
/// Python equivalent: MeshHelper.py lines 43-57
///
/// # Arguments
/// * `data` - Slice of Vector3 structs
///
/// # Returns
/// Vector of (f32, f32, f32) tuples
pub fn vector3_to_tuples(data: &[Vector3]) -> Vec<Tuple3f> {
    data.iter().map(|v| (v.x, v.y, v.z)).collect()
}

/// Converts Vector4 list to tuples
///
/// Python equivalent: MeshHelper.py lines 43-57
///
/// # Arguments
/// * `data` - Slice of Vector4 structs
///
/// # Returns
/// Vector of (f32, f32, f32, f32) tuples
pub fn vector4_to_tuples(data: &[Vector4]) -> Vec<Tuple4f> {
    data.iter().map(|v| (v.x, v.y, v.z, v.w)).collect()
}

/// Creates a zero-filled 1D array
///
/// Python equivalent: MeshHelper.py lines 60-67
///
/// # Arguments
/// * `size` - Number of elements
///
/// # Returns
/// Vector filled with zeros
pub fn zeros_1d(size: usize) -> Vec<f32> {
    vec![0.0; size]
}

/// Creates a 2D zero-filled array
///
/// Python equivalent: MeshHelper.py lines 60-67
///
/// # Arguments
/// * `rows` - Number of rows
/// * `cols` - Number of columns
///
/// # Returns
/// Nested vector filled with zeros
pub fn zeros_2d(rows: usize, cols: usize) -> Vec<Vec<f32>> {
    vec![vec![0.0; cols]; rows]
}

/// Normalizes a vector (makes length = 1)
///
/// Python equivalent: MeshHelper.py lines 70-75
///
/// # Arguments
/// * `vector` - Input vector components
///
/// # Returns
/// Normalized vector (length 1) or zero vector if input too small
///
/// # Example
/// ```ignore
/// let v = vec![3.0, 4.0];
/// let norm = normalize(&v);
/// // Returns: vec![0.6, 0.8] (length = 1.0)
/// ```
pub fn normalize(vector: &[f32]) -> Vec<f32> {
    let length: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if length > 0.00001 {
        let inv_norm = 1.0 / length;
        vector.iter().map(|v| v * inv_norm).collect()
    } else {
        vec![0.0; vector.len()]
    }
}

// =============================================================================
// SECTION 2: MESHHANDLER STRUCT
// =============================================================================

/// Enum for Mesh or SpriteRenderData source
///
/// Python uses Union[Mesh, SpriteRenderData] (line 79)
#[derive(Debug)]
pub enum MeshSource {
    Mesh(Mesh),
    Sprite(SpriteRenderData),
}

/// Main mesh processing handler
///
/// Python equivalent: MeshHandler class (lines 78-734)
///
/// This struct processes Unity mesh data from various formats and versions,
/// extracting vertex attributes into easy-to-use arrays.
#[derive(Debug)]
pub struct MeshHandler {
    // Source data
    pub src: MeshSource,
    pub endianess: Endian,
    pub version: (u32, u32, u32, u32),

    // Processed vertex data (Python lines 82-101)
    pub m_vertex_count: usize,
    pub m_vertices: Option<Vec<Tuple3f>>,
    pub m_normals: Option<Vec<Tuple3f>>, // Can be Tuple4f in some versions
    pub m_colors: Option<Vec<Tuple4f>>,
    pub m_uv0: Option<Vec<Tuple2f>>,
    pub m_uv1: Option<Vec<Tuple2f>>,
    pub m_uv2: Option<Vec<Tuple2f>>,
    pub m_uv3: Option<Vec<Tuple2f>>,
    pub m_uv4: Option<Vec<Tuple2f>>,
    pub m_uv5: Option<Vec<Tuple2f>>,
    pub m_uv6: Option<Vec<Tuple2f>>,
    pub m_uv7: Option<Vec<Tuple2f>>,
    pub m_tangents: Option<Vec<Tuple4f>>,
    pub m_bone_indices: Option<Vec<Tuple4i>>,
    pub m_bone_weights: Option<Vec<Tuple4f>>,
    pub m_index_buffer: Option<Vec<u8>>, // Raw bytes, processed later
    pub m_use_16bit_indices: bool,
}

// =============================================================================
// SECTION 3: IMPLEMENTATION
// =============================================================================

impl MeshHandler {
    /// Creates a new MeshHandler
    ///
    /// Python equivalent: __init__ (lines 103-116)
    ///
    /// # Arguments
    /// * `src` - Mesh or SpriteRenderData source
    /// * `version` - Optional Unity version (major, minor, patch, build)
    /// * `endianess` - Byte order (default: Little endian)
    ///
    /// # Returns
    /// New MeshHandler instance
    ///
    /// # Errors
    /// Returns error if version cannot be determined
    pub fn new(
        src: MeshSource,
        version: Option<(u32, u32, u32, u32)>,
        endianess: Option<Endian>,
    ) -> Result<Self, io::Error> {
        let endianess = endianess.unwrap_or(Endian::Little);

        // Determine version (Python lines 111-116)
        // Note: Rust's ObjectReaderTrait doesn't expose version field,
        // so version must be provided explicitly (architectural difference from Python)
        let version = version.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Version must be provided explicitly in Rust (ObjectReader.version not accessible through trait)"
            )
        })?;

        Ok(MeshHandler {
            src,
            endianess,
            version,
            m_vertex_count: 0,
            m_vertices: None,
            m_normals: None,
            m_colors: None,
            m_uv0: None,
            m_uv1: None,
            m_uv2: None,
            m_uv3: None,
            m_uv4: None,
            m_uv5: None,
            m_uv6: None,
            m_uv7: None,
            m_tangents: None,
            m_bone_indices: None,
            m_bone_weights: None,
            m_index_buffer: None,
            m_use_16bit_indices: true,
        })
    }

    /// Main processing entry point
    ///
    /// Python equivalent: process() (lines 118-198)
    ///
    /// This method:
    /// 1. Determines mesh format based on Unity version
    /// 2. Loads external resource data if needed
    /// 3. Copies direct mesh data
    /// 4. Parses vertex streams
    /// 5. Decompresses compressed meshes
    pub fn process(&mut self) -> Result<(), io::Error> {
        // Get mesh and vertex_data (Python lines 119-121)
        let mesh = &self.src;
        let vertex_data = match mesh {
            MeshSource::Mesh(m) => m.m_VertexData.as_ref(),
            MeshSource::Sprite(s) => s.m_VertexData.as_ref(),
        };

        let vertex_data = vertex_data
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No vertex data found"))?;

        // Determine channels and streams based on version (Python lines 123-150)
        let (m_channels, m_streams) =
            if self.version.0 < 4 {
                // Unity < 4.0: Use m_Streams_0_ through m_Streams_3_ (Python lines 126-140)
                let streams = vec![
                    vertex_data.m_Streams_0_.clone().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Missing m_Streams_0_")
                    })?,
                    vertex_data.m_Streams_1_.clone().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Missing m_Streams_1_")
                    })?,
                    vertex_data.m_Streams_2_.clone().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Missing m_Streams_2_")
                    })?,
                    vertex_data.m_Streams_3_.clone().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Missing m_Streams_3_")
                    })?,
                ];
                let channels = self.get_channels(&streams);
                (channels, streams)
            } else if self.version.0 == 4 {
                // Unity 4.x: Use m_Streams and m_Channels (Python lines 141-146)
                let streams = vertex_data.m_Streams.clone().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing m_Streams")
                })?;
                let channels = vertex_data.m_Channels.clone().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing m_Channels")
                })?;
                (channels, streams)
            } else {
                // Unity 5.0+: Use m_Channels and generate streams (Python lines 147-150)
                let channels = vertex_data.m_Channels.clone().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing m_Channels")
                })?;
                let vertex_count = vertex_data.m_VertexCount.unwrap_or(0);
                let streams = self.get_streams(&channels, vertex_count);
                (channels, streams)
            };

        // Load external stream data if needed (Python lines 152-165)
        // Note: This is currently not implemented due to architectural differences.
        // Python can modify vertex_data.m_DataSize in place, but Rust would need
        // mut access to self.src which conflicts with the immutable borrows above.
        // External resources should be loaded before creating MeshHandler.
        //
        // if let MeshSource::Mesh(mesh) = &self.src {
        //     if let Some(stream_data) = &mesh.m_StreamData {
        //         if !stream_data.path.is_empty() {
        //             let data = get_resource_data(...)?;
        //             // Cannot modify mesh here
        //         }
        //     }
        // }

        // Copy direct mesh data (Python lines 167-175)
        match &self.src {
            MeshSource::Mesh(mesh) => {
                // m_Use16BitIndices is Option<i32> where 0 = false
                if mesh.m_Use16BitIndices == Some(0) {
                    self.m_use_16bit_indices = false;
                }
                self.copy_from_mesh();
            }
            MeshSource::Sprite(_) => {
                self.copy_from_spriterenderdata();
            }
        }

        // Process index buffer (Python lines 177-189)
        if let Some(ref mut _index_buffer) = self.m_index_buffer {
            // Convert raw bytes to indices
            // Note: In Python this is done with struct.unpack
            // In Rust, we keep it as bytes and process in get_triangles()
            // This matches our implementation approach
        }

        // Read vertex data for Unity 3.5+ (Python lines 191-192)
        if self.version >= (3, 5, 0, 0) {
            self.read_vertex_data(&m_channels, &m_streams)?;
        }

        // Decompress compressed mesh for Unity 2.6+ (Python lines 194-195)
        if matches!(&self.src, MeshSource::Mesh(_)) && self.version >= (2, 6, 0, 0) {
            self.decompress_compressed_mesh()?;
        }

        // Set vertex count if not set (Python lines 197-198)
        if self.m_vertex_count == 0 {
            if let Some(ref vertices) = self.m_vertices {
                self.m_vertex_count = vertices.len();
            }
        }

        Ok(())
    }

    /// Copies data directly from Mesh struct fields
    ///
    /// Python equivalent: copy_from_mesh() (lines 200-248)
    ///
    /// This method copies vertex data that's stored directly in Mesh fields
    /// (as opposed to packed in m_VertexData streams). This happens in older
    /// Unity versions or when data isn't compressed.
    pub fn copy_from_mesh(&mut self) {
        let mesh = match &self.src {
            MeshSource::Mesh(m) => m,
            _ => return, // Not a mesh, skip
        };

        // Copy index buffer (Python line 206-207)
        if self.m_index_buffer.is_none() {
            if let Some(ref index_buffer) = mesh.m_IndexBuffer {
                if !index_buffer.is_empty() {
                    self.m_index_buffer = Some(index_buffer.clone());
                }
            }
        }

        // Copy vertices (Python lines 209-210)
        if self.m_vertices.is_none() {
            if let Some(ref vertices) = mesh.m_Vertices {
                if !vertices.is_empty() {
                    self.m_vertices = Some(vector3_to_tuples(vertices));
                }
            }
        }

        // Copy normals (Python lines 212-213)
        if self.m_normals.is_none() {
            if let Some(ref normals) = mesh.m_Normals {
                if !normals.is_empty() {
                    self.m_normals = Some(vector3_to_tuples(normals));
                }
            }
        }

        // Copy tangents (Python lines 215-216)
        if self.m_tangents.is_none() {
            if let Some(ref tangents) = mesh.m_Tangents {
                if !tangents.is_empty() {
                    self.m_tangents = Some(vector4_to_tuples(tangents));
                }
            }
        }

        // Copy UV0 (Python lines 218-219)
        if self.m_uv0.is_none() {
            if let Some(ref uv) = mesh.m_UV {
                if !uv.is_empty() {
                    self.m_uv0 = Some(vector2_to_tuples(uv));
                }
            }
        }

        // Copy UV1 (Python lines 221-222)
        if self.m_uv1.is_none() {
            if let Some(ref uv1) = mesh.m_UV1 {
                if !uv1.is_empty() {
                    self.m_uv1 = Some(vector2_to_tuples(uv1));
                }
            }
        }

        // Copy colors (Python lines 224-228)
        // Note: Despite ColorRGBA being typed as float, Unity stores color values
        // in 0-255 range, so we must normalize by dividing by 255.0
        if self.m_colors.is_none() {
            if let Some(ref colors) = mesh.m_Colors {
                if !colors.is_empty() {
                    self.m_colors = Some(
                        colors
                            .iter()
                            .map(|c| (c.r / 255.0, c.g / 255.0, c.b / 255.0, c.a / 255.0))
                            .collect(),
                    );
                }
            }
        }

        // Copy bone weights and indices from skin data (Python lines 230-248)
        if self.m_bone_weights.is_none() {
            if let Some(ref skin) = mesh.m_Skin {
                if !skin.is_empty() {
                    let count = skin.len();
                    let mut bone_weights = Vec::with_capacity(count);
                    let mut bone_indices = Vec::with_capacity(count);

                    for influence in skin {
                        // Extract bone indices (Python lines 237-242)
                        bone_indices.push((
                            influence.boneIndex_0_.unwrap_or(0),
                            influence.boneIndex_1_.unwrap_or(0),
                            influence.boneIndex_2_.unwrap_or(0),
                            influence.boneIndex_3_.unwrap_or(0),
                        ));

                        // Extract bone weights (Python lines 243-248)
                        bone_weights.push((
                            influence.weight_0_.unwrap_or(0.0),
                            influence.weight_1_.unwrap_or(0.0),
                            influence.weight_2_.unwrap_or(0.0),
                            influence.weight_3_.unwrap_or(0.0),
                        ));
                    }

                    self.m_bone_indices = Some(bone_indices);
                    self.m_bone_weights = Some(bone_weights);
                }
            }
        }
    }

    /// Copies data from SpriteRenderData
    ///
    /// Python equivalent: copy_from_spriterenderdata() (lines 250-269)
    ///
    /// Sprites store their mesh data differently than regular meshes.
    /// This method extracts vertex positions and UVs from SpriteVertex structs.
    pub fn copy_from_spriterenderdata(&mut self) {
        let sprite = match &self.src {
            MeshSource::Sprite(s) => s,
            _ => return, // Not a sprite, skip
        };

        // Copy index buffer (Python lines 255-259)
        // Try m_IndexBuffer first, then fall back to indices
        if self.m_index_buffer.is_none() {
            if let Some(ref index_buffer) = sprite.m_IndexBuffer {
                if !index_buffer.is_empty() {
                    self.m_index_buffer = Some(index_buffer.clone());
                }
            } else if let Some(ref indices) = sprite.indices {
                if !indices.is_empty() {
                    // Convert u16 indices to u8 bytes (matching Python behavior)
                    let mut bytes = Vec::with_capacity(indices.len() * 2);
                    for &idx in indices {
                        bytes.extend_from_slice(&idx.to_le_bytes());
                    }
                    self.m_index_buffer = Some(bytes);
                }
            }
        }

        // Copy vertices and UV0 from SpriteVertex array (Python lines 261-266)
        if self.m_vertices.is_none() {
            if let Some(ref vertices) = sprite.vertices {
                if !vertices.is_empty() {
                    // Extract positions (Python line 263)
                    self.m_vertices = Some(
                        vertices
                            .iter()
                            .filter_map(|v| v.pos.as_ref().map(|pos| (pos.x, pos.y, pos.z)))
                            .collect(),
                    );

                    // Extract UVs if present (Python lines 265-266)
                    // Check first vertex to see if UVs exist
                    if vertices[0].uv.is_some() {
                        self.m_uv0 = Some(
                            vertices
                                .iter()
                                .filter_map(|v| v.uv.as_ref().map(|uv| (uv.x, uv.y)))
                                .collect(),
                        );
                    }
                }
            }
        }

        // Note: m_BindPose is commented out in Python (lines 268-269)
        // so we skip it here as well
    }

    /// Calculates stream layout for Unity 5+
    ///
    /// Python equivalent: get_streams() (lines 271-298)
    ///
    /// In Unity 5+, channels define what data exists, and streams define
    /// how that data is packed in memory. This method calculates the stream
    /// layout from channel definitions.
    ///
    /// # Arguments
    /// * `channels` - Channel definitions (what attributes exist)
    /// * `vertex_count` - Number of vertices
    ///
    /// # Returns
    /// Vector of StreamInfo structs defining memory layout
    fn get_streams(&self, channels: &[ChannelInfo], vertex_count: u32) -> Vec<StreamInfo> {
        // Find the highest stream index (Python line 274)
        let stream_count = channels.iter().filter_map(|c| c.stream).max().unwrap_or(0) as usize + 1;

        let mut streams = Vec::with_capacity(stream_count);
        let mut offset = 0u32;

        // For each stream (Python lines 277-297)
        for s in 0..stream_count {
            let mut channel_mask = 0u32;
            let mut stride = 0u32;

            // Calculate mask and stride for this stream (Python lines 280-285)
            for (chn, channel) in channels.iter().enumerate() {
                let stream_val = channel.stream.unwrap_or(0);
                let dimension_val = channel.dimension.unwrap_or(0);

                if stream_val == s as u8 && dimension_val > 0 {
                    // Set bit in channel mask (Python line 283)
                    channel_mask |= 1 << chn;

                    // Add to stride (Python lines 284-285)
                    let component_size = self.get_channel_component_size(channel);
                    stride += (dimension_val & 0xF) as u32 * component_size;
                }
            }

            // Create StreamInfo (Python lines 287-295)
            let mut stream_info = StreamInfo::default();
            stream_info.channelMask = Some(channel_mask);
            stream_info.offset = Some(offset);
            stream_info.stride = Some(stride);
            stream_info.dividerOp = Some(0);
            stream_info.frequency = Some(0);
            streams.push(stream_info);

            // Update offset for next stream (Python lines 296-297)
            offset += vertex_count * stride;
            // Align to 16 bytes
            offset = (offset + 15) & !15;
        }

        streams
    }

    /// Calculates channel layout for Unity <5
    ///
    /// Python equivalent: get_channels() (lines 300-338)
    ///
    /// In Unity <5, streams define the memory layout, and channels must be
    /// derived from the stream channel masks. This reverses the Unity 5+ process.
    ///
    /// # Arguments
    /// * `streams` - Stream definitions from older Unity versions
    ///
    /// # Returns
    /// Vector of ChannelInfo structs
    fn get_channels(&self, streams: &[StreamInfo]) -> Vec<ChannelInfo> {
        // Create 6 default channels (Python lines 301-309)
        // 0: Vertex, 1: Normal, 2: Color, 3: UV0, 4: UV1, 5: Tangent
        let mut channels = vec![ChannelInfo::default(); 6];

        // Process each stream (Python lines 310-337)
        for (s, stream) in streams.iter().enumerate() {
            let channel_mask = stream.channelMask.unwrap_or(0);
            let mut offset = 0u8;

            // Check each of the 6 possible channels (Python lines 313-336)
            for i in 0..6 {
                // Check if this channel is enabled in the mask (Python line 314)
                if (channel_mask & (1 << i)) != 0 {
                    let channel = &mut channels[i];
                    channel.stream = Some(s as u8);
                    channel.offset = Some(offset);

                    // Set format and dimension based on channel type (Python lines 318-333)
                    match i {
                        0 | 1 => {
                            // 0 = kShaderChannelVertex, 1 = kShaderChannelNormal
                            channel.format = Some(0); // kChannelFormatFloat
                            channel.dimension = Some(3);
                        }
                        2 => {
                            // kShaderChannelColor
                            channel.format = Some(2); // kChannelFormatColor
                            channel.dimension = Some(4);
                        }
                        3 | 4 => {
                            // 3 = kShaderChannelTexCoord0, 4 = kShaderChannelTexCoord1
                            channel.format = Some(0); // kChannelFormatFloat
                            channel.dimension = Some(2);
                        }
                        5 => {
                            // kShaderChannelTangent
                            channel.format = Some(0); // kChannelFormatFloat
                            channel.dimension = Some(4);
                        }
                        _ => {} // Should never happen (we only iterate 0-5)
                    }

                    // Update offset for next channel (Python lines 335-336)
                    let component_size = self.get_channel_component_size(channel);
                    offset += channel.dimension.unwrap_or(0) * component_size as u8;
                }
            }
        }

        channels
    }

    /// Parses vertex data from binary streams
    ///
    /// Python equivalent: read_vertex_data() (lines 340-420)
    ///
    /// This is the most complex parsing method. It:
    /// 1. Reads binary data from m_VertexData.m_DataSize
    /// 2. Interprets using stream/channel layout
    /// 3. Converts binary formats to f32 arrays
    /// 4. Assigns to appropriate fields
    fn read_vertex_data(
        &mut self,
        channels: &[ChannelInfo],
        streams: &[StreamInfo],
    ) -> Result<(), io::Error> {
        // Get vertex data from mesh (Python lines 343-347)
        let vertex_data = match &self.src {
            MeshSource::Mesh(mesh) => {
                if let Some(ref vd) = mesh.m_VertexData {
                    vd
                } else {
                    return Ok(()); // No vertex data
                }
            }
            MeshSource::Sprite(sprite) => {
                if let Some(ref vd) = sprite.m_VertexData {
                    vd
                } else {
                    return Ok(()); // No vertex data
                }
            }
        };

        self.m_vertex_count = vertex_data.m_VertexCount.unwrap_or(0) as usize;
        let vertex_count = vertex_data.m_VertexCount.unwrap_or(0);
        // Clone data_size to avoid borrow issues
        let data_size = vertex_data.m_DataSize.clone().unwrap_or_default();

        // Process each channel (Python lines 350-420)
        for (chn, channel) in channels.iter().enumerate() {
            // Skip disabled channels (Python lines 351-352)
            if channel.dimension.unwrap_or(0) == 0 {
                continue;
            }

            let stream = &streams[channel.stream.unwrap_or(0) as usize];

            // Check if this channel is in the stream's mask (Python lines 359-360)
            if (stream.channelMask.unwrap_or(0) & (1 << chn)) == 0 {
                continue;
            }

            // Special case for color in Unity < 2018 (Python lines 361-370)
            let mut channel_to_use = channel.clone();
            if self.version.0 < 2018 && chn == 2 && channel.format.unwrap_or(0) == 2 {
                // kShaderChannelColor && kChannelFormatColor
                channel_to_use.dimension = Some(4);
            }

            // Get format info (Python lines 372-377)
            let component_byte_size = self.get_channel_component_size(&channel_to_use);
            let swap_bytes = self.endianess == Endian::Little && component_byte_size > 1;
            let channel_dimension = (channel_to_use.dimension.unwrap_or(0) & 0xF) as usize;

            // Allocate buffer for unpacked data (Python lines 391-393)
            let total_bytes =
                vertex_count as usize * channel_dimension * component_byte_size as usize;
            let mut component_bytes = vec![0u8; total_bytes];

            // Unpack vertex data (Python lines 395-412)
            // This extracts interleaved data into a flat array
            let vertex_base_offset =
                stream.offset.unwrap_or(0) + channel_to_use.offset.unwrap_or(0) as u32;

            for v in 0..vertex_count as usize {
                let vertex_offset = vertex_base_offset + stream.stride.unwrap_or(0) * v as u32;

                for d in 0..channel_dimension {
                    let component_offset = vertex_offset + component_byte_size * d as u32;
                    let component_data_dest =
                        component_byte_size as usize * (v * channel_dimension + d);

                    // Copy component bytes
                    let src_start = component_offset as usize;
                    let src_end = src_start + component_byte_size as usize;

                    if src_end <= data_size.len() {
                        let mut buff = data_size[src_start..src_end].to_vec();

                        // Swap bytes if needed for endianness (Python line 407-408)
                        if swap_bytes {
                            buff.reverse();
                        }

                        // Copy to destination
                        let dest_start = component_data_dest;
                        let dest_end = dest_start + component_byte_size as usize;
                        component_bytes[dest_start..dest_end].copy_from_slice(&buff);
                    }
                }
            }

            // Convert binary data to floats (Python lines 414-418)
            let component_data =
                self.unpack_component_data(&component_bytes, &channel_to_use, channel_dimension)?;

            // Assign to appropriate field (Python line 420)
            self.assign_channel_vertex_data(chn as u8, component_data);
        }

        Ok(())
    }

    /// Unpacks binary component data to floats
    ///
    /// Converts raw bytes to f32 values based on format type
    fn unpack_component_data(
        &self,
        bytes: &[u8],
        channel: &ChannelInfo,
        dimension: usize,
    ) -> Result<Vec<Vec<f32>>, io::Error> {
        let dtype = self.get_channel_dtype(channel);
        let component_size = self.get_channel_component_size(channel) as usize;
        let count = bytes.len() / component_size;

        // Convert bytes to values based on format
        let mut values = Vec::with_capacity(count);

        for i in 0..count {
            let start = i * component_size;
            let end = start + component_size;
            let chunk = &bytes[start..end];

            let value = match dtype {
                "f" => {
                    // f32 - 4 bytes
                    f32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                }
                "e" => {
                    // f16 - 2 bytes (half float)
                    // Convert to f32 using half crate or manual conversion
                    let bits = u16::from_be_bytes([chunk[0], chunk[1]]);
                    half::f16::from_bits(bits).to_f32()
                }
                "B" => {
                    // u8 - 1 byte, normalize to 0-1
                    chunk[0] as f32 / 255.0
                }
                "b" => {
                    // i8 - 1 byte signed, normalize to -1 to 1
                    (chunk[0] as i8) as f32 / 127.0
                }
                "H" => {
                    // u16 - 2 bytes, normalize to 0-1
                    u16::from_be_bytes([chunk[0], chunk[1]]) as f32 / 65535.0
                }
                "h" => {
                    // i16 - 2 bytes signed, normalize to -1 to 1
                    i16::from_be_bytes([chunk[0], chunk[1]]) as f32 / 32767.0
                }
                "I" => {
                    // u32 - 4 bytes
                    u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32
                }
                "i" => {
                    // i32 - 4 bytes signed
                    i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32
                }
                _ => 0.0,
            };

            values.push(value);
        }

        // Convert flat array to tuples (Python line 418)
        Ok(flat_list_to_tuples(&values, dimension)
            .into_iter()
            .map(|v| v.into_iter().collect())
            .collect())
    }

    /// Assigns parsed channel data to struct fields
    ///
    /// Python equivalent: assign_channel_vertex_data() (lines 422-476)
    ///
    /// Maps channel indices to struct fields. Channel mapping varies by Unity version.
    fn assign_channel_vertex_data(&mut self, channel: u8, component_data: Vec<Vec<f32>>) {
        // Convert Vec<Vec<f32>> to appropriate tuple types
        if self.version.0 >= 2018 {
            // Unity 2018+ channel mapping (Python lines 423-454)
            match channel {
                0 => {
                    // kShaderChannelVertex
                    self.m_vertices = Some(
                        component_data
                            .into_iter()
                            .map(|v| (v[0], v[1], v[2]))
                            .collect(),
                    );
                }
                1 => {
                    // kShaderChannelNormal
                    self.m_normals = Some(
                        component_data
                            .into_iter()
                            .map(|v| (v[0], v[1], v[2]))
                            .collect(),
                    );
                }
                2 => {
                    // kShaderChannelTangent
                    self.m_tangents = Some(
                        component_data
                            .into_iter()
                            .map(|v| {
                                let x = v.get(0).copied().unwrap_or(0.0);
                                let y = v.get(1).copied().unwrap_or(0.0);
                                let z = v.get(2).copied().unwrap_or(0.0);
                                let w = v.get(3).copied().unwrap_or(1.0);
                                (x, y, z, w)
                            })
                            .collect(),
                    );
                }
                3 => {
                    // kShaderChannelColor
                    self.m_colors = Some(
                        component_data
                            .into_iter()
                            .map(|v| {
                                let r = v.get(0).copied().unwrap_or(1.0);
                                let g = v.get(1).copied().unwrap_or(1.0);
                                let b = v.get(2).copied().unwrap_or(1.0);
                                let a = v.get(3).copied().unwrap_or(1.0);
                                (r, g, b, a)
                            })
                            .collect(),
                    );
                }
                4 => {
                    // kShaderChannelTexCoord0
                    self.m_uv0 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                5 => {
                    // kShaderChannelTexCoord1
                    self.m_uv1 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                6 => {
                    // kShaderChannelTexCoord2
                    self.m_uv2 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                7 => {
                    // kShaderChannelTexCoord3
                    self.m_uv3 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                8 => {
                    // kShaderChannelTexCoord4
                    self.m_uv4 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                9 => {
                    // kShaderChannelTexCoord5
                    self.m_uv5 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                10 => {
                    // kShaderChannelTexCoord6
                    self.m_uv6 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                11 => {
                    // kShaderChannelTexCoord7
                    self.m_uv7 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                12 => {
                    // kShaderChannelBlendWeight (2018.2+)
                    self.m_bone_weights = Some(
                        component_data
                            .into_iter()
                            .map(|v| {
                                let w0 = v.get(0).copied().unwrap_or(0.0);
                                let w1 = v.get(1).copied().unwrap_or(0.0);
                                let w2 = v.get(2).copied().unwrap_or(0.0);
                                let w3 = v.get(3).copied().unwrap_or(0.0);
                                (w0, w1, w2, w3)
                            })
                            .collect(),
                    );
                }
                13 => {
                    // kShaderChannelBlendIndices (2018.2+)
                    self.m_bone_indices = Some(
                        component_data
                            .into_iter()
                            .map(|v| {
                                let i0 = v.get(0).copied().unwrap_or(0.0) as i32;
                                let i1 = v.get(1).copied().unwrap_or(0.0) as i32;
                                let i2 = v.get(2).copied().unwrap_or(0.0) as i32;
                                let i3 = v.get(3).copied().unwrap_or(0.0) as i32;
                                (i0, i1, i2, i3)
                            })
                            .collect(),
                    );
                }
                _ => {
                    eprintln!("Warning: Unknown channel {}", channel);
                }
            }
        } else {
            // Unity < 2018 channel mapping (Python lines 456-476)
            match channel {
                0 => {
                    // kShaderChannelVertex
                    self.m_vertices = Some(
                        component_data
                            .into_iter()
                            .map(|v| (v[0], v[1], v[2]))
                            .collect(),
                    );
                }
                1 => {
                    // kShaderChannelNormal
                    self.m_normals = Some(
                        component_data
                            .into_iter()
                            .map(|v| (v[0], v[1], v[2]))
                            .collect(),
                    );
                }
                2 => {
                    // kShaderChannelColor
                    self.m_colors = Some(
                        component_data
                            .into_iter()
                            .map(|v| {
                                let r = v.get(0).copied().unwrap_or(1.0);
                                let g = v.get(1).copied().unwrap_or(1.0);
                                let b = v.get(2).copied().unwrap_or(1.0);
                                let a = v.get(3).copied().unwrap_or(1.0);
                                (r, g, b, a)
                            })
                            .collect(),
                    );
                }
                3 => {
                    // kShaderChannelTexCoord0
                    self.m_uv0 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                4 => {
                    // kShaderChannelTexCoord1
                    self.m_uv1 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                5 => {
                    if self.version.0 >= 5 {
                        // kShaderChannelTexCoord2
                        self.m_uv2 =
                            Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                    } else {
                        // kShaderChannelTangent
                        self.m_tangents = Some(
                            component_data
                                .into_iter()
                                .map(|v| {
                                    let x = v.get(0).copied().unwrap_or(0.0);
                                    let y = v.get(1).copied().unwrap_or(0.0);
                                    let z = v.get(2).copied().unwrap_or(0.0);
                                    let w = v.get(3).copied().unwrap_or(1.0);
                                    (x, y, z, w)
                                })
                                .collect(),
                        );
                    }
                }
                6 => {
                    // kShaderChannelTexCoord3
                    self.m_uv3 = Some(component_data.into_iter().map(|v| (v[0], v[1])).collect());
                }
                7 => {
                    // kShaderChannelTangent
                    self.m_tangents = Some(
                        component_data
                            .into_iter()
                            .map(|v| {
                                let x = v.get(0).copied().unwrap_or(0.0);
                                let y = v.get(1).copied().unwrap_or(0.0);
                                let z = v.get(2).copied().unwrap_or(0.0);
                                let w = v.get(3).copied().unwrap_or(1.0);
                                (x, y, z, w)
                            })
                            .collect(),
                    );
                }
                _ => {
                    eprintln!("Warning: Unknown channel {}", channel);
                }
            }
        }
    }

    /// Gets struct format string for channel
    ///
    /// Python equivalent: get_channel_dtype() (lines 478-489)
    ///
    /// Returns the Python struct format character for the channel's data type.
    /// This is used to determine component size.
    pub fn get_channel_dtype(&self, channel: &ChannelInfo) -> &'static str {
        // Version-specific format interpretation (Python lines 479-487)
        let format = channel.format.unwrap_or(0);

        if self.version.0 < 2017 {
            // Use VertexChannelFormat (Python lines 480-481)
            match format {
                0 => "f", // kChannelFormatFloat = 4 bytes
                1 => "e", // kChannelFormatFloat16 = 2 bytes (half float)
                2 => "B", // kChannelFormatColor = 1 byte (unsigned)
                3 => "B", // kChannelFormatByte = 1 byte (unsigned)
                4 => "I", // kChannelFormatUInt32 = 4 bytes
                _ => "f", // Default to float
            }
        } else if self.version.0 < 2019 {
            // Use VertexFormat2017 (Python lines 482-484)
            match format {
                0 => "f",  // kVertexFormatFloat = 4 bytes
                1 => "e",  // kVertexFormatFloat16 = 2 bytes
                2 => "B",  // kVertexFormatColor = 1 byte
                3 => "B",  // kVertexFormatUNorm8 = 1 byte
                4 => "b",  // kVertexFormatSNorm8 = 1 byte (signed)
                5 => "H",  // kVertexFormatUNorm16 = 2 bytes (unsigned short)
                6 => "h",  // kVertexFormatSNorm16 = 2 bytes (signed short)
                7 => "B",  // kVertexFormatUInt8 = 1 byte
                8 => "b",  // kVertexFormatSInt8 = 1 byte
                9 => "H",  // kVertexFormatUInt16 = 2 bytes
                10 => "h", // kVertexFormatSInt16 = 2 bytes
                11 => "I", // kVertexFormatUInt32 = 4 bytes
                12 => "i", // kVertexFormatSInt32 = 4 bytes
                _ => "f",  // Default to float
            }
        } else {
            // Use VertexFormat (Python lines 486-487)
            match format {
                0 => "f",  // kVertexFormatFloat = 4 bytes
                1 => "e",  // kVertexFormatFloat16 = 2 bytes
                2 => "B",  // kVertexFormatUNorm8 = 1 byte
                3 => "b",  // kVertexFormatSNorm8 = 1 byte
                4 => "H",  // kVertexFormatUNorm16 = 2 bytes
                5 => "h",  // kVertexFormatSNorm16 = 2 bytes
                6 => "B",  // kVertexFormatUInt8 = 1 byte
                7 => "b",  // kVertexFormatSInt8 = 1 byte
                8 => "H",  // kVertexFormatUInt16 = 2 bytes
                9 => "h",  // kVertexFormatSInt16 = 2 bytes
                10 => "I", // kVertexFormatUInt32 = 4 bytes
                11 => "i", // kVertexFormatSInt32 = 4 bytes
                _ => "f",  // Default to float
            }
        }
    }

    /// Gets byte size of channel component
    ///
    /// Python equivalent: get_channel_component_size() (lines 491-493)
    ///
    /// Returns the size in bytes of one component of the channel's data type.
    pub fn get_channel_component_size(&self, channel: &ChannelInfo) -> u32 {
        // Get format character and map to size (Python line 493)
        let dtype = self.get_channel_dtype(channel);
        match dtype {
            "f" => 4, // float (f32)
            "e" => 2, // half float (f16)
            "B" => 1, // unsigned byte (u8)
            "b" => 1, // signed byte (i8)
            "H" => 2, // unsigned short (u16)
            "h" => 2, // signed short (i16)
            "I" => 4, // unsigned int (u32)
            "i" => 4, // signed int (i32)
            _ => 4,   // Default to 4 bytes
        }
    }

    /// Decompresses compressed mesh data
    ///
    /// Python equivalent: decompress_compressed_mesh() (lines 495-655)
    ///
    /// Unity can compress vertex data using PackedBitVector format.
    /// This method unpacks:
    /// - Vertices
    /// - Normals
    /// - Tangents
    /// - Colors
    /// - UVs
    /// - Bone weights
    /// - Triangles/indices
    fn decompress_compressed_mesh(&mut self) -> Result<(), io::Error> {
        // Get CompressedMesh from source (Python lines 500)
        let m_compressed_mesh = match &self.src {
            MeshSource::Mesh(mesh) => mesh.m_CompressedMesh.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Mesh does not have compressed mesh data",
                )
            })?,
            MeshSource::Sprite(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "SpriteRenderData does not have compressed mesh data",
                ))
            }
        };

        // Vertex Count (Python lines 502-504)
        self.m_vertex_count = (m_compressed_mesh
            .m_Vertices
            .as_ref()
            .map_or(0, |v| v.m_NumItems.unwrap_or(0))
            / 3) as usize;
        let m_vertex_count = self.m_vertex_count;

        // Vertices (Python lines 506-507)
        if m_compressed_mesh
            .m_Vertices
            .as_ref()
            .map_or(0, |v| v.m_NumItems.unwrap_or(0))
            > 0
        {
            let unpacked = Self::unpack_packed_floats(
                m_compressed_mesh.m_Vertices.as_ref().unwrap(),
                0,
                None,
                Some(&[3]),
            )?;
            self.m_vertices = Some(unpacked.iter().map(|v| (v[0], v[1], v[2])).collect());
        }

        // UV (Python lines 510-543)
        if m_compressed_mesh
            .m_UV
            .as_ref()
            .map_or(0, |v| v.m_NumItems.unwrap_or(0))
            > 0
        {
            // Check if m_UVInfo exists (Python line 512)
            if let Some(m_uv_info) = m_compressed_mesh.m_UVInfo {
                if m_uv_info != 0 {
                    // Constants (Python lines 513-516)
                    const K_INFO_BITS_PER_UV: u32 = 4;
                    const K_UV_DIMENSION_MASK: u32 = 3;
                    const K_UV_CHANNEL_EXISTS: u32 = 4;
                    const K_MAX_TEX_COORD_SHADER_CHANNELS: usize = 8;

                    let mut uv_src_offset = 0;

                    // Process each UV channel (Python lines 520-532)
                    for uv_channel in 0..K_MAX_TEX_COORD_SHADER_CHANNELS {
                        let mut tex_coord_bits =
                            m_uv_info >> ((uv_channel as u32) * K_INFO_BITS_PER_UV);
                        tex_coord_bits &= (1 << K_INFO_BITS_PER_UV) - 1;

                        if (tex_coord_bits & K_UV_CHANNEL_EXISTS) != 0 {
                            let uv_dim = 1 + (tex_coord_bits & K_UV_DIMENSION_MASK) as usize;
                            let m_uv = Self::unpack_packed_floats(
                                m_compressed_mesh.m_UV.as_ref().ok_or_else(|| {
                                    std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "m_UV is None",
                                    )
                                })?,
                                uv_src_offset,
                                Some(m_vertex_count * uv_dim),
                                Some(&[uv_dim]),
                            )?;

                            // Assign to appropriate UV channel (Python line 531)
                            // Convert Vec<Vec<f32>> to Vec<Tuple2f>, padding with 0.0 if needed
                            let uv_tuples: Vec<Tuple2f> = m_uv
                                .iter()
                                .map(|v| {
                                    (
                                        v.get(0).copied().unwrap_or(0.0),
                                        v.get(1).copied().unwrap_or(0.0),
                                    )
                                })
                                .collect();

                            match uv_channel {
                                0 => self.m_uv0 = Some(uv_tuples),
                                1 => self.m_uv1 = Some(uv_tuples),
                                2 => self.m_uv2 = Some(uv_tuples),
                                3 => self.m_uv3 = Some(uv_tuples),
                                4 => self.m_uv4 = Some(uv_tuples),
                                5 => self.m_uv5 = Some(uv_tuples),
                                6 => self.m_uv6 = Some(uv_tuples),
                                7 => self.m_uv7 = Some(uv_tuples),
                                _ => {}
                            }

                            uv_src_offset = uv_dim * m_vertex_count;
                        }
                    }
                } else {
                    // No m_UVInfo - handle legacy UV format (Python lines 534-543)
                    let uv0_data = Self::unpack_packed_floats(
                        m_compressed_mesh.m_UV.as_ref().ok_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::InvalidData, "m_UV is None")
                        })?,
                        0,
                        Some(m_vertex_count * 2),
                        Some(&[2]),
                    )?;
                    self.m_uv0 = Some(uv0_data.iter().map(|v| (v[0], v[1])).collect());

                    // Check if UV1 exists (Python lines 537-543)
                    let uv_num_items = m_compressed_mesh
                        .m_UV
                        .as_ref()
                        .and_then(|v| v.m_NumItems)
                        .unwrap_or(0);
                    if uv_num_items >= (m_vertex_count * 4) as u32 {
                        let uv1_data = Self::unpack_packed_floats(
                            m_compressed_mesh.m_UV.as_ref().unwrap(),
                            m_vertex_count * 2,
                            Some(m_vertex_count * 2),
                            Some(&[2]),
                        )?;
                        self.m_uv1 = Some(uv1_data.iter().map(|v| (v[0], v[1])).collect());
                    }
                }
            } else {
                // No m_UVInfo - handle legacy UV format (same as above)
                let uv0_data = Self::unpack_packed_floats(
                    m_compressed_mesh.m_UV.as_ref().ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "m_UV is None")
                    })?,
                    0,
                    Some(m_vertex_count * 2),
                    Some(&[2]),
                )?;
                self.m_uv0 = Some(uv0_data.iter().map(|v| (v[0], v[1])).collect());

                let uv_num_items = m_compressed_mesh
                    .m_UV
                    .as_ref()
                    .and_then(|v| v.m_NumItems)
                    .unwrap_or(0);
                if uv_num_items >= (m_vertex_count * 4) as u32 {
                    let uv1_data = Self::unpack_packed_floats(
                        m_compressed_mesh.m_UV.as_ref().unwrap(),
                        m_vertex_count * 2,
                        Some(m_vertex_count * 2),
                        Some(&[2]),
                    )?;
                    self.m_uv1 = Some(uv1_data.iter().map(|v| (v[0], v[1])).collect());
                }
            }
        }

        // BindPose (Python lines 546-555)
        if self.version.0 < 5 {
            // Unity 5.0 and below
            if let Some(ref m_bind_poses) = m_compressed_mesh.m_BindPoses {
                if m_bind_poses.m_NumItems.unwrap_or(0) > 0 {
                    // Unpack 4x4 matrices (Python lines 549-555)
                    let _m_bind_pose =
                        Self::unpack_packed_floats_raw(m_bind_poses, 0, None, Some(&[4, 4]))?;
                    // Note: m_BindPose is not stored in MeshHandler, just unpacked
                }
            }
        }

        // Normals (Python lines 558-573)
        if m_compressed_mesh
            .m_Normals
            .as_ref()
            .and_then(|v| v.m_NumItems)
            .unwrap_or(0)
            > 0
        {
            let normal_data = Self::unpack_packed_floats(
                m_compressed_mesh.m_Normals.as_ref().unwrap(),
                0,
                None,
                Some(&[2]),
            )?;
            let signs = Self::unpack_packed_ints(
                m_compressed_mesh.m_NormalSigns.as_ref().unwrap(),
                0,
                None,
                None,
            )?;

            let mut normals = Vec::with_capacity(m_vertex_count);

            // Reconstruct 3D normals from 2D + sign (Python lines 563-573)
            for (normal_2d, sign) in normal_data.iter().zip(signs.iter()) {
                let x = normal_2d[0];
                let y = normal_2d[1];
                let zsqr = 1.0 - x * x - y * y;

                let mut z = if zsqr >= 0.0 {
                    zsqr.sqrt()
                } else {
                    // Normalize if invalid (Python line 571)
                    let normalized = normalize(&[x, y, 0.0]);
                    normals.push((normalized[0], normalized[1], normalized[2]));
                    continue;
                };

                // Apply sign to z component (Python lines 572-573)
                if *sign == 0 {
                    z *= -1.0;
                }

                normals.push((x, y, z));
            }

            self.m_normals = Some(normals);
        }

        // Tangents (Python lines 576-594)
        if m_compressed_mesh
            .m_Tangents
            .as_ref()
            .and_then(|v| v.m_NumItems)
            .unwrap_or(0)
            > 0
        {
            let tangent_data = Self::unpack_packed_floats(
                m_compressed_mesh.m_Tangents.as_ref().unwrap(),
                0,
                None,
                Some(&[2]),
            )?;
            let signs = Self::unpack_packed_ints(
                m_compressed_mesh.m_TangentSigns.as_ref().unwrap(),
                0,
                None,
                None,
            )?;

            let mut tangents = Vec::with_capacity(m_vertex_count);

            // Reconstruct 4D tangents from 2D + 2 signs (Python lines 580-594)
            for (tangent_2d, sign_pair) in tangent_data.iter().zip(signs.chunks(2)) {
                let sign_z = sign_pair[0];
                let sign_w = sign_pair.get(1).copied().unwrap_or(1);

                let x = tangent_2d[0];
                let y = tangent_2d[1];
                let zsqr = 1.0 - x * x - y * y;

                let mut z;
                let w;

                if zsqr >= 0.0 {
                    z = zsqr.sqrt();
                } else {
                    // Normalize if invalid (Python line 590)
                    let normalized = normalize(&[x, y, 0.0]);
                    z = normalized[2];
                }

                // Apply signs (Python lines 591-593)
                if sign_z == 0 {
                    z = -z;
                }
                w = if sign_w > 0 { 1.0 } else { -1.0 };

                tangents.push((x, y, z, w));
            }

            self.m_tangents = Some(tangents);
        }

        // FloatColors (Python lines 597-600)
        if self.version.0 >= 5 {
            // Unity 5.0 and up
            if let Some(ref m_float_colors) = m_compressed_mesh.m_FloatColors {
                if m_float_colors.m_NumItems.unwrap_or(0) > 0 {
                    let colors_data =
                        Self::unpack_packed_floats(m_float_colors, 0, None, Some(&[4]))?;
                    self.m_colors = Some(
                        colors_data
                            .iter()
                            .map(|v| (v[0], v[1], v[2], v[3]))
                            .collect(),
                    );
                }
            }
        }

        // Skin (bone weights and indices) (Python lines 602-639)
        if m_compressed_mesh
            .m_Weights
            .as_ref()
            .and_then(|v| v.m_NumItems)
            .unwrap_or(0)
            > 0
        {
            let weights_data = Self::unpack_packed_ints(
                m_compressed_mesh.m_Weights.as_ref().unwrap(),
                0,
                None,
                None,
            )?;
            // Normalize weights (Python line 604)
            let weights_data: Vec<f32> = weights_data.iter().map(|&w| w as f32 / 31.0).collect();
            let bone_indices_data = Self::unpack_packed_ints(
                m_compressed_mesh.m_BoneIndices.as_ref().unwrap(),
                0,
                None,
                None,
            )?;

            let mut vertex_index = 0;
            let mut j = 0;
            let mut sum = 0.0;

            let mut bone_weights = vec![(0.0, 0.0, 0.0, 0.0); m_vertex_count];
            let mut bone_indices = vec![(0, 0, 0, 0); m_vertex_count];

            let mut bone_indices_iter = bone_indices_data.iter();

            // Process weights and indices (Python lines 615-639)
            for weight in weights_data.iter() {
                if vertex_index >= m_vertex_count {
                    break;
                }

                let bone_index = *bone_indices_iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Bone indices exhausted")
                })?;

                // Assign weight and bone index (Python lines 617-618)
                match j {
                    0 => {
                        bone_weights[vertex_index].0 = *weight;
                        bone_indices[vertex_index].0 = bone_index as i32;
                    }
                    1 => {
                        bone_weights[vertex_index].1 = *weight;
                        bone_indices[vertex_index].1 = bone_index as i32;
                    }
                    2 => {
                        bone_weights[vertex_index].2 = *weight;
                        bone_indices[vertex_index].2 = bone_index as i32;
                    }
                    3 => {
                        bone_weights[vertex_index].3 = *weight;
                        bone_indices[vertex_index].3 = bone_index as i32;
                    }
                    _ => {}
                }

                j += 1;
                sum += weight;

                // Weights add up to 1.0 - move to next vertex (Python lines 624-630)
                if sum >= 1.0 {
                    vertex_index += 1;
                    j = 0;
                    sum = 0.0;
                }
                // Read 3 weights but they don't add to 1.0 - calculate 4th weight (Python lines 633-639)
                else if j == 3 {
                    bone_weights[vertex_index].3 = 1.0 - sum;
                    let last_bone_index = *bone_indices_iter.next().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Bone indices exhausted for 4th weight",
                        )
                    })?;
                    bone_indices[vertex_index].3 = last_bone_index as i32;

                    vertex_index += 1;
                    j = 0;
                    sum = 0.0;
                }
            }

            self.m_bone_weights = Some(bone_weights);
            self.m_bone_indices = Some(bone_indices);
        }

        // IndexBuffer (Python lines 642-643)
        if m_compressed_mesh
            .m_Triangles
            .as_ref()
            .and_then(|v| v.m_NumItems)
            .unwrap_or(0)
            > 0
        {
            let triangles = Self::unpack_packed_ints(
                m_compressed_mesh.m_Triangles.as_ref().unwrap(),
                0,
                None,
                None,
            )?;
            // Convert to bytes for consistency with copy_from_mesh
            let mut index_buffer = Vec::new();
            for idx in triangles {
                // Store as u32 bytes (little-endian)
                index_buffer.extend_from_slice(&idx.to_le_bytes());
            }
            self.m_index_buffer = Some(index_buffer);
        }

        // Color (integer RGBA format) (Python lines 645-654)
        if let Some(ref m_colors_packed) = m_compressed_mesh.m_Colors {
            if m_colors_packed.m_NumItems.unwrap_or(0) > 0 {
                let rgba_colors = Self::unpack_packed_ints(m_colors_packed, 0, None, None)?;
                // Unpack RGBA from u32 (Python lines 647-654)
                let colors: Vec<Tuple4f> = rgba_colors
                    .iter()
                    .map(|&rgba| {
                        (
                            ((rgba >> 24) & 0xFF) as f32 / 255.0,
                            ((rgba >> 16) & 0xFF) as f32 / 255.0,
                            ((rgba >> 8) & 0xFF) as f32 / 255.0,
                            (rgba & 0xFF) as f32 / 255.0,
                        )
                    })
                    .collect();
                self.m_colors = Some(colors);
            }
        }

        Ok(())
    }

    /// Helper: Unpacks PackedBitVector as floats and returns as Vec<Vec<f32>>
    ///
    /// Python equivalent: unpack_floats() from PackedBitVector.py
    fn unpack_packed_floats(
        packed: &PackedBitVector,
        start: usize,
        count: Option<usize>,
        shape: Option<&[usize]>,
    ) -> Result<Vec<Vec<f32>>, io::Error> {
        let bit_size = packed.m_BitSize.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PackedBitVector missing m_BitSize",
            )
        })?;
        let range = packed.m_Range.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PackedBitVector missing m_Range",
            )
        })?;
        let start_val = packed.m_Start.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PackedBitVector missing m_Start",
            )
        })?;

        let num_items = count.unwrap_or(packed.m_NumItems.unwrap_or(0) as usize);

        let data = packed.m_Data.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "PackedBitVector missing m_Data")
        })?;

        let result = unpack_floats(
            data,
            num_items,
            bit_size as u32,
            range,
            start_val,
            start,
            shape,
        );

        // Convert ReshapedData to Vec<Vec<f32>>
        match result {
            ReshapedData::Flat(data) => Ok(vec![data]),
            ReshapedData::TwoLayer(data) => Ok(data),
            ReshapedData::FourLayer(data) => {
                // Flatten 4-layer to 2-layer
                Ok(data.into_iter().flatten().flatten().collect())
            }
        }
    }

    /// Helper: Unpacks PackedBitVector as floats (raw ReshapedData for matrices)
    fn unpack_packed_floats_raw(
        packed: &PackedBitVector,
        start: usize,
        count: Option<usize>,
        shape: Option<&[usize]>,
    ) -> Result<ReshapedData<f32>, io::Error> {
        let bit_size = packed.m_BitSize.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PackedBitVector missing m_BitSize",
            )
        })?;
        let range = packed.m_Range.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PackedBitVector missing m_Range",
            )
        })?;
        let start_val = packed.m_Start.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PackedBitVector missing m_Start",
            )
        })?;

        let num_items = count.unwrap_or(packed.m_NumItems.unwrap_or(0) as usize);

        let data = packed.m_Data.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "PackedBitVector missing m_Data")
        })?;

        Ok(unpack_floats(
            data,
            num_items,
            bit_size as u32,
            range,
            start_val,
            start,
            shape,
        ))
    }

    /// Helper: Unpacks PackedBitVector as ints
    ///
    /// Python equivalent: unpack_ints() from PackedBitVector.py
    fn unpack_packed_ints(
        packed: &PackedBitVector,
        start: usize,
        count: Option<usize>,
        shape: Option<&[usize]>,
    ) -> Result<Vec<u32>, io::Error> {
        let bit_size = packed.m_BitSize.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "PackedBitVector missing m_BitSize",
            )
        })?;

        let num_items = count.unwrap_or(packed.m_NumItems.unwrap_or(0) as usize);

        let data = packed.m_Data.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "PackedBitVector missing m_Data")
        })?;

        let result = unpack_ints(data, num_items, bit_size as u32, start, shape);

        // Flatten to Vec<u32>
        Ok(result.flatten())
    }

    /// Extracts triangle indices organized by submesh
    ///
    /// Python equivalent: get_triangles() (lines 657-734)
    ///
    /// Returns a list of submeshes, where each submesh contains triangles.
    /// Each triangle is a vec of 3 vertex indices.
    ///
    /// # Topology Handling
    ///
    /// - Triangles: Direct extraction in groups of 3
    /// - TriangleStrip: Converts strip to triangles with winding flip-flop
    /// - Quads: Splits each quad into 2 triangles
    /// - Lines/Points: Returns error (not drawable as triangles)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No index buffer exists
    /// - No submesh data available
    /// - Topology is Lines or Points
    pub fn get_triangles(&self) -> Result<Vec<Vec<Vec<u32>>>, io::Error> {
        // Ensure index buffer exists (Python line 658)
        let index_buffer = self.m_index_buffer.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "No index buffer available")
        })?;

        // Get source mesh data (Python line 662)
        let m_submeshes = match &self.src {
            MeshSource::Mesh(mesh) => mesh.m_SubMeshes.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Mesh has no submesh data")
            })?,
            MeshSource::Sprite(sprite) => sprite.m_SubMeshes.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "SpriteRenderData has no submesh data",
                )
            })?,
        };

        let mut submeshes: Vec<Vec<Vec<u32>>> = Vec::new();

        // Process each submesh (Python line 663)
        for m_submesh in m_submeshes {
            // Calculate first index position (Python lines 664-666)
            let mut first_index = (m_submesh.firstByte.unwrap_or(0) / 2) as usize;
            if !self.m_use_16bit_indices {
                first_index /= 2;
            }

            let index_count = m_submesh.indexCount.unwrap_or(0) as usize;

            // Determine topology (Python line 669)
            let topology = if let Some(topology_val) = m_submesh.topology {
                MeshTopology::from_u32(topology_val as u32)
            } else {
                // Legacy: use isTriStrip field (pre-Unity 4)
                if m_submesh.isTriStrip.unwrap_or(0) != 0 {
                    MeshTopology::TriangleStrip
                } else {
                    MeshTopology::Triangles
                }
            };

            let triangles: Vec<Vec<u32>>;

            // Process based on topology (Python lines 673-711)
            match topology {
                MeshTopology::Triangles => {
                    // Direct triangle extraction (Python lines 674-675)
                    let indices = self.read_indices(index_buffer, first_index, index_count)?;
                    triangles = indices.chunks(3).map(|chunk| chunk.to_vec()).collect();
                }

                MeshTopology::TriangleStrip => {
                    // Convert triangle strip to triangles (Python lines 676-696)
                    // Unity < 4.0 also uses TriangleStrip (Python line 677)
                    let indices = self.read_indices(index_buffer, first_index, index_count)?;
                    let mut tri_list = Vec::new();

                    for i in 0..(index_count - 2) {
                        let a = indices[i];
                        let b = indices[i + 1];
                        let c = indices[i + 2];

                        // Skip degenerate triangles (Python lines 686-687)
                        if a == b || a == c || b == c {
                            continue;
                        }

                        // Winding flip-flop for strips (Python lines 690-693)
                        if i & 1 != 0 {
                            tri_list.push(vec![b, a, c]);
                        } else {
                            tri_list.push(vec![a, b, c]);
                        }
                    }

                    triangles = tri_list;
                }

                MeshTopology::Quads => {
                    // Convert quads to triangles (Python lines 698-707)
                    let indices = self.read_indices(index_buffer, first_index, index_count)?;
                    let mut tri_list = Vec::with_capacity((index_count / 2) as usize);

                    for i in (0..index_count).step_by(4) {
                        if i + 3 >= index_count {
                            break;
                        }
                        let a = indices[i];
                        let b = indices[i + 1];
                        let c = indices[i + 2];
                        let d = indices[i + 3];

                        // Split quad into 2 triangles (Python lines 705-706)
                        tri_list.push(vec![a, b, c]);
                        tri_list.push(vec![a, c, d]);
                    }

                    triangles = tri_list;
                }

                MeshTopology::Lines | MeshTopology::LineStrip | MeshTopology::Points => {
                    // Lines and points cannot be converted to triangles (Python lines 709-711)
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Failed getting triangles. Submesh topology is {:?}.",
                            topology
                        ),
                    ));
                }

                MeshTopology::Unknown(val) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unknown mesh topology value: {}", val),
                    ));
                }
            }

            submeshes.push(triangles);
        }

        Ok(submeshes)
    }

    /// Helper: Reads indices from index buffer
    ///
    /// Handles both 16-bit and 32-bit index formats
    fn read_indices(
        &self,
        index_buffer: &[u8],
        first_index: usize,
        count: usize,
    ) -> Result<Vec<u32>, io::Error> {
        let mut indices = Vec::with_capacity(count);

        if self.m_use_16bit_indices {
            // 16-bit indices (2 bytes each)
            let byte_offset = first_index * 2;
            for i in 0..count {
                let offset = byte_offset + i * 2;
                if offset + 1 >= index_buffer.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Index buffer too short for 16-bit indices",
                    ));
                }
                let idx = u16::from_le_bytes([index_buffer[offset], index_buffer[offset + 1]]);
                indices.push(idx as u32);
            }
        } else {
            // 32-bit indices (4 bytes each)
            let byte_offset = first_index * 4;
            for i in 0..count {
                let offset = byte_offset + i * 4;
                if offset + 3 >= index_buffer.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Index buffer too short for 32-bit indices",
                    ));
                }
                let idx = u32::from_le_bytes([
                    index_buffer[offset],
                    index_buffer[offset + 1],
                    index_buffer[offset + 2],
                    index_buffer[offset + 3],
                ]);
                indices.push(idx);
            }
        }

        Ok(indices)
    }
}
