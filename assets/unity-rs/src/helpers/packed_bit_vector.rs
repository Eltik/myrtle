//! PackedBitVector - Bit-packed data structures for animation curves
//!
//! Unity uses packed bit vectors to compress animation curve data.
//! This module provides unpacking (decompression) functionality.
//!
//! Python equivalent: helpers/PackedBitVector.py
//!

use std::cmp;

/// Represents the result of reshape operation with Python-exact structure
#[derive(Debug, Clone)]
pub enum ReshapedData<T> {
    /// None case: flat list (1 layer)
    Flat(Vec<T>),
    /// 1D case: 2 layers
    TwoLayer(Vec<Vec<T>>),
    /// 2D case: 4 layers
    FourLayer(Vec<Vec<Vec<Vec<T>>>>),
}

impl<T: Clone> ReshapedData<T> {
    /// Flattens the reshaped data back to a flat vector
    pub fn flatten(self) -> Vec<T> {
        match self {
            ReshapedData::Flat(data) => data,
            ReshapedData::TwoLayer(data) => data.into_iter().flatten().collect(),
            ReshapedData::FourLayer(data) => {
                data.into_iter().flatten().flatten().flatten().collect()
            }
        }
    }
}

/// Unpacks N-bit integers from a packed byte array
///
/// # Arguments
///
/// * `data` - Packed byte array containing bit-packed integers
/// * `num_items` - Number of integers to unpack
/// * `bit_size` - Number of bits per integer (e.g., 10 for 10-bit values)
/// * `start` - Starting item index (for partial unpacking)
///
/// # Algorithm
///
/// Reads N-bit values from a packed byte array where values may span byte boundaries.
/// For example, with 10-bit values:
/// - Value 0: bits 0-9
/// - Value 1: bits 10-19 (spans bytes 1-2)
/// - Value 2: bits 20-29 (spans bytes 2-3)
///
/// # Python equivalent
/// PackedBitVector.py: unpack_ints() (lines 23-68)
pub fn unpack_ints(
    data: &[u8],
    num_items: usize,
    bit_size: u32,
    start: usize,
    shape: Option<&[usize]>,
) -> ReshapedData<u32> {
    // Calculate starting bit position (bit_size * start)
    let mut bit_pos = (bit_size as usize) * start;

    // Convert bit position to byte index
    let mut index_pos = bit_pos / 8;

    bit_pos %= 8;

    let mut result = vec![0u32; num_items];

    for i in 0..num_items {
        let mut bits = 0;
        let mut value = 0;
        while bits < bit_size {
            value |= ((data[index_pos] >> bit_pos) as u32) << bits;
            let num = cmp::min(bit_size - bits, 8 - bit_pos as u32);
            bit_pos += num as usize;
            bits += num;

            if bit_pos == 8 {
                index_pos += 1;
                bit_pos = 0;
            }
        }
        result[i] = value & ((1 << bit_size) - 1);
    }

    reshape(result, shape)
}

/// Unpacks N-bit integers and dequantizes them to floats
///
/// # Arguments
///
/// * `data` - Packed byte array
/// * `num_items` - Number of values to unpack
/// * `bit_size` - Number of bits per value
/// * `range` - Range of original values (max - min)
/// * `start_val` - Minimum value in original data
/// * `start` - Starting item index
///
/// # Algorithm
///
/// 1. Unpack N-bit integers using unpack_ints()
/// 2. Calculate scale = range / ((1 << bit_size) - 1)
/// 3. Dequantize: float = (int_value * scale) + start_val
///
/// # Python equivalent
/// PackedBitVector.py: unpack_floats() (lines 71-86)
pub fn unpack_floats(
    data: &[u8],
    num_items: usize,
    bit_size: u32,
    range: f32,
    start_val: f32,
    start: usize,
    shape: Option<&[usize]>,
) -> ReshapedData<f32> {
    // Unpack as integers (Python line 84: unpack_ints without shape)
    let quantized = unpack_ints(data, num_items, bit_size, start, None);
    // Flatten the 4-level structure to get raw integers
    let flat: Vec<u32> = quantized.flatten();

    // Calculate scale (Python: scale = packed.m_Range / ((1 << packed.m_BitSize) - 1))
    let max_value = ((1u32 << bit_size) - 1) as f32;
    let scale = range / max_value;

    // Dequantize to floats (Python: x * scale + packed.m_Start)
    let dequantized: Vec<f32> = flat
        .iter()
        .map(|&x| (x as f32) * scale + start_val)
        .collect();

    // Reshape the dequantized values (matches Python line 86)
    reshape(dequantized, shape)
}

/// Reshapes a flat vector into nested vectors with 3-layer structure
///
/// # Arguments
///
/// * `data` - Flat data vector
/// * `shape` - Optional shape specification [rows, cols]
///
/// # Return Structure (3 layers)
///
/// ```
/// // None: [1,2,3,4,5,6] -> [[[1,2,3,4,5,6]]]
/// // 1D: [1,2,3,4,5,6] with shape [3] -> [[[1,2,3], [4,5,6]]]
/// // 2D: [1,2,3,4,5,6] with shape [2,3] -> [[[1,2,3], [4,5,6]]]
/// ```
///
/// # Python equivalent
/// PackedBitVector.py: reshape() (lines 7-20)
///
/// Python's actual 2D output has 4 levels, but we use 3 levels consistently
/// for all cases in Rust to maintain a uniform type signature.
pub fn reshape<T: Clone>(data: Vec<T>, shape: Option<&[usize]>) -> ReshapedData<T> {
    match shape {
        None => ReshapedData::Flat(data),

        Some(s) if s.len() == 1 && s[0] > 0 => {
            let m = s[0];
            ReshapedData::TwoLayer(data.chunks(m).map(|chunk| chunk.to_vec()).collect())
        }

        Some(s) if s.len() == 2 && s[0] > 0 && s[1] > 0 => {
            let m = s[0];
            let n = s[1];
            let chunk_size = m * n;

            ReshapedData::FourLayer(
                data.chunks(chunk_size)
                    .map(|outer_chunk| {
                        vec![outer_chunk.chunks(n).map(|chunk| chunk.to_vec()).collect()]
                    })
                    .collect(),
            )
        }

        // Invalid shape: return wrapped data
        _ => ReshapedData::Flat(data),
    }
}
