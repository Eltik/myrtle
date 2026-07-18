//! Minimal Unity `Mesh` (class 43) decoder + 4x4 transform math, used to
//! rasterize dynchar background scenes.
//!
//! Only the pieces needed for flat 2D compositing are decoded: vertex
//! positions (channel 0) and UV0 (channel 4) plus the triangle index buffer.
//! Normals/tangents/colors/bones are ignored.

use std::collections::HashMap;

use base64::Engine;
use serde_json::Value;

/// Decoded triangle geometry in mesh-local space.
pub struct MeshData {
    /// Vertex positions (x, y, z), mesh-local.
    pub positions: Vec<[f32; 3]>,
    /// UV0 per vertex (u, v). Same length as `positions`.
    pub uvs: Vec<[f32; 2]>,
    /// Vertex colour (RGBA, 0..1) per vertex; white when absent. Arknights fx
    /// meshes bake per-vertex opacity/falloff here, so it must modulate the
    /// sampled texture.
    pub colors: Vec<[f32; 4]>,
    /// Flat triangle list (indices into `positions`), length is a multiple of 3.
    pub indices: Vec<u32>,
}

/// Unity's built-in Quad primitive (used when a `MeshFilter` has `m_Mesh == 0`):
/// a 1x1 square in the XY plane centred on the origin, UVs 0..1.
#[must_use]
pub fn unit_quad() -> MeshData {
    MeshData {
        positions: vec![
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.5, 0.5, 0.0],
            [-0.5, 0.5, 0.0],
        ],
        uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        colors: vec![[1.0; 4]; 4],
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

/// Byte size of a Unity vertex-channel format code.
const fn format_size(format: i64) -> usize {
    match format {
        0 | 6 | 7 | 11 => 4, // Float32 / UInt32 / SInt32
        1 | 4 | 5 | 8 | 9 | 10 => 2, // Float16 / (U|S)Norm16 / (U|S)Int16
        _ => 1,              // (U|S)Norm8 / (U|S)Int8
    }
}

/// Decode a half-precision float.
fn half_to_f32(h: u16) -> f32 {
    let sign = u32::from(h >> 15) << 31;
    let exp = u32::from((h >> 10) & 0x1f);
    let mant = u32::from(h & 0x3ff);
    let bits = if exp == 0 {
        if mant == 0 {
            sign
        } else {
            // subnormal
            let mut e = -1i32;
            let mut m = mant;
            while m & 0x400 == 0 {
                m <<= 1;
                e -= 1;
            }
            m &= 0x3ff;
            sign | (((e + 127 - 15) as u32) << 23) | (m << 13)
        }
    } else if exp == 0x1f {
        sign | 0x7f80_0000 | (mant << 13)
    } else {
        sign | ((exp + 127 - 15) << 23) | (mant << 13)
    };
    f32::from_bits(bits)
}

fn read_scalar(buf: &[u8], off: usize, format: i64) -> f32 {
    match format {
        0 => {
            let b: [u8; 4] = buf.get(off..off + 4).and_then(|s| s.try_into().ok()).unwrap_or([0; 4]);
            f32::from_le_bytes(b)
        }
        1 => {
            let b: [u8; 2] = buf.get(off..off + 2).and_then(|s| s.try_into().ok()).unwrap_or([0; 2]);
            half_to_f32(u16::from_le_bytes(b))
        }
        // UNorm8 (vertex colours)
        2 => f32::from(buf.get(off).copied().unwrap_or(0)) / 255.0,
        // UNorm16
        4 => {
            let b: [u8; 2] = buf.get(off..off + 2).and_then(|s| s.try_into().ok()).unwrap_or([0; 2]);
            f32::from(u16::from_le_bytes(b)) / 65535.0
        }
        _ => 0.0,
    }
}

/// Parse a Unity `Mesh` object into flat triangle geometry.
///
/// `resources` supplies external vertex/index buffers referenced via
/// `m_StreamData`. Returns `None` if the mesh is unreadable (compressed,
/// unsupported channel format, missing buffers, non-triangle topology).
#[must_use]
pub fn parse_mesh(mesh: &Value, resources: &HashMap<String, Vec<u8>>) -> Option<MeshData> {
    // Compressed meshes are not supported.
    if mesh
        .get("m_CompressedMesh")
        .and_then(|c| c.get("m_Vertices"))
        .and_then(|v| v.get("m_NumItems"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0)
        > 0
    {
        return None;
    }

    let vd = mesh.get("m_VertexData")?;
    let vertex_count = vd.get("m_VertexCount").and_then(serde_json::Value::as_u64)? as usize;
    if vertex_count == 0 {
        return None;
    }
    let channels = vd.get("m_Channels").and_then(|c| c.as_array())?;

    // Vertex buffer: inline base64 (TypelessData surfaces as `m_DataSize`), or
    // external via m_StreamData.
    let vbuf: Vec<u8> = {
        let inline = vd
            .get("m_DataSize")
            .and_then(|d| d.as_str())
            .filter(|s| !s.is_empty())
            .and_then(|s| base64::engine::general_purpose::STANDARD.decode(s).ok());
        match inline {
            Some(b) => b,
            None => read_stream_data(mesh.get("m_StreamData"), resources)?,
        }
    };

    // Per-stream stride = max(channel.offset + channel.size) over that stream.
    let mut stream_stride: HashMap<i64, usize> = HashMap::new();
    for ch in channels {
        let dim = ch.get("dimension").and_then(serde_json::Value::as_i64).unwrap_or(0);
        if dim <= 0 {
            continue;
        }
        let stream = ch.get("stream").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let offset = ch.get("offset").and_then(serde_json::Value::as_i64).unwrap_or(0) as usize;
        let format = ch.get("format").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let end = offset + dim as usize * format_size(format);
        let e = stream_stride.entry(stream).or_insert(0);
        *e = (*e).max(end);
    }
    // Stream start offsets: streams are concatenated, each aligned to 16 bytes.
    let mut streams: Vec<i64> = stream_stride.keys().copied().collect();
    streams.sort_unstable();
    let mut stream_start: HashMap<i64, usize> = HashMap::new();
    let mut cursor = 0usize;
    for s in &streams {
        stream_start.insert(*s, cursor);
        let block = stream_stride[s] * vertex_count;
        cursor += block.div_ceil(16) * 16;
    }

    let channel = |i: usize| -> Option<(i64, usize, i64)> {
        let ch = channels.get(i)?;
        let dim = ch.get("dimension").and_then(serde_json::Value::as_i64).unwrap_or(0);
        if dim <= 0 {
            return None;
        }
        let stream = ch.get("stream").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let offset = ch.get("offset").and_then(serde_json::Value::as_i64).unwrap_or(0) as usize;
        let format = ch.get("format").and_then(serde_json::Value::as_i64).unwrap_or(0);
        Some((stream, offset, format))
    };

    // Channel 0 = Position (required), channel 4 = TexCoord0 (fallback 0,0).
    let (p_stream, p_off, p_fmt) = channel(0)?;
    let p_start = *stream_start.get(&p_stream)?;
    let p_stride = *stream_stride.get(&p_stream)?;

    let uv = channel(4);
    let color = channel(3); // Color

    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let base = p_start + i * p_stride + p_off;
        positions.push([
            read_scalar(&vbuf, base, p_fmt),
            read_scalar(&vbuf, base + format_size(p_fmt), p_fmt),
            read_scalar(&vbuf, base + 2 * format_size(p_fmt), p_fmt),
        ]);
        let uv0 = uv.and_then(|(s, o, f)| {
            let start = *stream_start.get(&s)?;
            let stride = *stream_stride.get(&s)?;
            let b = start + i * stride + o;
            Some([read_scalar(&vbuf, b, f), read_scalar(&vbuf, b + format_size(f), f)])
        });
        uvs.push(uv0.unwrap_or([0.0, 0.0]));
        let col = color.and_then(|(s, o, f)| {
            let start = *stream_start.get(&s)?;
            let stride = *stream_stride.get(&s)?;
            let b = start + i * stride + o;
            let fs = format_size(f);
            Some([
                read_scalar(&vbuf, b, f),
                read_scalar(&vbuf, b + fs, f),
                read_scalar(&vbuf, b + 2 * fs, f),
                read_scalar(&vbuf, b + 3 * fs, f),
            ])
        });
        colors.push(col.unwrap_or([1.0; 4]));
    }

    // Index buffer (array of u8) → triangle list, honoring submesh ranges.
    let ibuf: Vec<u8> = mesh
        .get("m_IndexBuffer")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_u64().map(|n| n as u8)).collect())
        .unwrap_or_default();
    let index_16 = mesh.get("m_IndexFormat").and_then(serde_json::Value::as_i64).unwrap_or(0) == 0;
    let read_index = |byte: usize| -> u32 {
        if index_16 {
            let b: [u8; 2] = ibuf.get(byte..byte + 2).and_then(|s| s.try_into().ok()).unwrap_or([0; 2]);
            u32::from(u16::from_le_bytes(b))
        } else {
            let b: [u8; 4] = ibuf.get(byte..byte + 4).and_then(|s| s.try_into().ok()).unwrap_or([0; 4]);
            u32::from_le_bytes(b)
        }
    };
    let istep = if index_16 { 2 } else { 4 };

    let mut indices = Vec::new();
    if let Some(submeshes) = mesh.get("m_SubMeshes").and_then(|v| v.as_array()) {
        for sm in submeshes {
            // topology 0 = triangles; skip line/point strips.
            if sm.get("topology").and_then(serde_json::Value::as_i64).unwrap_or(0) != 0 {
                continue;
            }
            let first_byte = sm.get("firstByte").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
            let index_count = sm.get("indexCount").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
            let base_vertex = sm.get("baseVertex").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
            for k in 0..index_count {
                let idx = read_index(first_byte + k * istep) + base_vertex;
                if (idx as usize) < vertex_count {
                    indices.push(idx);
                }
            }
        }
    }
    // Fallback: whole buffer as one triangle list.
    if indices.is_empty() {
        let n = ibuf.len() / istep;
        for k in 0..n {
            let idx = read_index(k * istep);
            if (idx as usize) < vertex_count {
                indices.push(idx);
            }
        }
    }
    if indices.len() < 3 {
        return None;
    }

    Some(MeshData { positions, uvs, colors, indices })
}

fn read_stream_data(sd: Option<&Value>, resources: &HashMap<String, Vec<u8>>) -> Option<Vec<u8>> {
    let sd = sd?;
    let size = sd.get("size").and_then(serde_json::Value::as_u64)? as usize;
    let offset = sd.get("offset").and_then(serde_json::Value::as_u64)? as usize;
    let path = sd.get("path").and_then(|p| p.as_str())?;
    if size == 0 || path.is_empty() {
        return None;
    }
    let filename = path.rsplit('/').next().unwrap_or(path);
    let res = resources.get(filename)?;
    res.get(offset..offset + size).map(<[u8]>::to_vec)
}

/// Row-major 4x4 transform matrix.
#[derive(Clone, Copy)]
pub struct Mat4(pub [[f32; 4]; 4]);

impl Mat4 {
    #[must_use]
    pub fn identity() -> Self {
        let mut m = [[0.0; 4]; 4];
        for (i, row) in m.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        Self(m)
    }

    /// Translate-Rotate-Scale from a Unity local transform.
    #[must_use]
    pub fn trs(pos: [f32; 3], quat: [f32; 4], scale: [f32; 3]) -> Self {
        let [x, y, z, w] = quat;
        // Rotation matrix from quaternion.
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);
        let r = [
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy)],
            [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx)],
            [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy)],
        ];
        let mut m = [[0.0; 4]; 4];
        for i in 0..3 {
            m[i][0] = r[i][0] * scale[0];
            m[i][1] = r[i][1] * scale[1];
            m[i][2] = r[i][2] * scale[2];
            m[i][3] = pos[i];
        }
        m[3][3] = 1.0;
        Self(m)
    }

    /// Matrix product `self * rhs`.
    #[must_use]
    pub fn mul(&self, rhs: &Self) -> Self {
        let mut m = [[0.0; 4]; 4];
        for (i, row) in m.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let mut s = 0.0;
                for k in 0..4 {
                    s += self.0[i][k] * rhs.0[k][j];
                }
                *cell = s;
            }
        }
        Self(m)
    }

    /// Transform a point (implicit w = 1), ignoring perspective divide.
    #[must_use]
    pub fn point(&self, p: [f32; 3]) -> [f32; 3] {
        let m = &self.0;
        [
            m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
            m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
            m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
        ]
    }
}
