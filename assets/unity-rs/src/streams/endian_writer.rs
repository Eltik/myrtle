use crate::math::{Color, Matrix4x4, Quaternion, Rectangle, Vector2, Vector3, Vector4};
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use std::io;

pub use crate::streams::endian::Endian;

/// Trait for writing binary data with endianness support
///
/// Provides methods for writing primitive types, strings, and Unity-specific
/// types to binary data. All methods handle endianness conversion automatically.
///
/// # Required Methods
///
/// Implementations must provide:
/// - `write()` - Core byte writing operation
/// - `position()`, `set_position()` - Position tracking
/// - `endian()`, `set_endian()` - Endianness control
/// - `len()` - Total bytes written
///
/// # Default Methods
///
/// The trait provides default implementations for all writing methods
/// (primitives, strings, Unity types) that build on the required methods.
///
/// # Examples
///
/// ```
/// use unity_rs::streams::endian_writer::{BinaryWriter, EndianBinaryWriter, Endian};
///
/// let mut writer = EndianBinaryWriter::new(Endian::Little);
///
/// writer.write_i32(42).unwrap();
/// writer.write_f32(3.14).unwrap();
/// writer.write_bool(true).unwrap();
///
/// let bytes = writer.to_bytes();
/// assert_eq!(bytes.len(), 9); // 4 + 4 + 1 bytes
/// ```
pub trait BinaryWriter {
    // Core write operation
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error>;

    // Position management
    fn position(&self) -> usize;
    fn set_position(&mut self, pos: usize);

    // Endianness
    fn endian(&self) -> Endian;
    fn set_endian(&mut self, endian: Endian);

    // Length (total bytes written)
    fn len(&self) -> usize;

    // Math methods
    fn write_u8(&mut self, value: u8) -> Result<(), io::Error> {
        self.write(&[value])?;
        Ok(())
    }
    fn write_i8(&mut self, value: i8) -> Result<(), io::Error> {
        self.write(&[value as u8])?;
        Ok(())
    }

    fn write_u16(&mut self, value: u16) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_u16::<LittleEndian>(value)?,
            Endian::Big => buffer.write_u16::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    fn write_i16(&mut self, value: i16) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_i16::<LittleEndian>(value)?,
            Endian::Big => buffer.write_i16::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_u32::<LittleEndian>(value)?,
            Endian::Big => buffer.write_u32::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    fn write_i32(&mut self, value: i32) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_i32::<LittleEndian>(value)?,
            Endian::Big => buffer.write_i32::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    fn write_u64(&mut self, value: u64) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_u64::<LittleEndian>(value)?,
            Endian::Big => buffer.write_u64::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    fn write_i64(&mut self, value: i64) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_i64::<LittleEndian>(value)?,
            Endian::Big => buffer.write_i64::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    fn write_f32(&mut self, value: f32) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_f32::<LittleEndian>(value)?,
            Endian::Big => buffer.write_f32::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    fn write_f64(&mut self, value: f64) -> Result<(), io::Error> {
        let mut buffer = Vec::new();

        match self.endian() {
            Endian::Little => buffer.write_f64::<LittleEndian>(value)?,
            Endian::Big => buffer.write_f64::<BigEndian>(value)?,
        }

        self.write(&buffer)?;
        Ok(())
    }

    // Boolean
    fn write_bool(&mut self, value: bool) -> Result<(), io::Error> {
        let u8_value = match value {
            true => 1,
            false => 0,
        };

        self.write_u8(u8_value)?;
        Ok(())
    }

    // Write multiple values
    fn write_vector2(&mut self, value: &Vector2) -> Result<(), io::Error> {
        self.write_f32(value.x)?;
        self.write_f32(value.y)?;
        Ok(())
    }

    fn write_vector3(&mut self, value: &Vector3) -> Result<(), io::Error> {
        self.write_f32(value.x)?;
        self.write_f32(value.y)?;
        self.write_f32(value.z)?;
        Ok(())
    }

    fn write_vector4(&mut self, value: &Vector4) -> Result<(), io::Error> {
        self.write_f32(value.x)?;
        self.write_f32(value.y)?;
        self.write_f32(value.z)?;
        self.write_f32(value.w)?;
        Ok(())
    }

    fn write_quaternion(&mut self, value: &Quaternion) -> Result<(), io::Error> {
        self.write_f32(value.x)?;
        self.write_f32(value.y)?;
        self.write_f32(value.z)?;
        self.write_f32(value.w)?;
        Ok(())
    }

    fn write_rectangle(&mut self, value: &Rectangle) -> Result<(), io::Error> {
        self.write_f32(value.x)?;
        self.write_f32(value.y)?;
        self.write_f32(value.width)?;
        self.write_f32(value.height)?;
        Ok(())
    }

    fn write_color(&mut self, value: &Color) -> Result<(), io::Error> {
        self.write_f32(value.r)?;
        self.write_f32(value.g)?;
        self.write_f32(value.b)?;
        self.write_f32(value.a)?;
        Ok(())
    }

    fn write_matrix4x4(&mut self, value: &Matrix4x4) -> Result<(), io::Error> {
        for &element in &value.data {
            self.write_f32(element)?;
        }
        Ok(())
    }

    // Utility methods
    fn write_bytes(&mut self, value: &[u8]) -> Result<(), io::Error> {
        self.write(value)?;
        Ok(())
    }

    fn write_string_to_null(&mut self, value: &str) -> Result<(), io::Error> {
        self.write(value.as_bytes())?;
        self.write_u8(0)?;
        Ok(())
    }

    fn write_align(&mut self) -> Result<(), io::Error> {
        let position = self.position();
        let padding = (4 - (position % 4)) % 4;

        for _ in 0..padding {
            self.write_u8(0)?;
        }

        Ok(())
    }

    fn align_stream(&mut self, alignment: usize) -> Result<(), io::Error> {
        let position = self.position();
        let padding = (alignment - (position % alignment)) % alignment;

        for _ in 0..padding {
            self.write_u8(0)?;
        }

        Ok(())
    }

    fn write_aligned_string(&mut self, value: &str) -> Result<(), io::Error> {
        let bstring = value.as_bytes();
        self.write_i32(bstring.len() as i32)?;
        self.write(bstring)?;
        self.align_stream(4)?;

        Ok(())
    }

    fn write_color_uint(&mut self, value: &Color) -> Result<(), io::Error> {
        self.write_u8((value.r * 255.0) as u8)?;
        self.write_u8((value.g * 255.0) as u8)?;
        self.write_u8((value.b * 255.0) as u8)?;
        self.write_u8((value.a * 255.0) as u8)?;

        Ok(())
    }

    fn write_byte_array(&mut self, value: &[u8]) -> Result<(), io::Error> {
        self.write_i32(value.len() as i32)?;
        self.write(value)?;
        Ok(())
    }

    fn write_boolean_array(&mut self, value: &[bool]) -> Result<(), io::Error> {
        self.write_i32(value.len() as i32)?;
        for &item in value {
            self.write_bool(item)?;
        }
        Ok(())
    }

    fn write_u_short_array(&mut self, value: &[u16]) -> Result<(), io::Error> {
        self.write_i32(value.len() as i32)?;
        for &item in value {
            self.write_u16(item)?;
        }
        Ok(())
    }

    fn write_int_array(&mut self, value: &[i32], write_length: bool) -> Result<(), io::Error> {
        if write_length {
            self.write_i32(value.len() as i32)?;
        }
        for &item in value {
            self.write_i32(item)?;
        }
        Ok(())
    }

    fn write_u_int_array(&mut self, value: &[u32], write_length: bool) -> Result<(), io::Error> {
        if write_length {
            self.write_i32(value.len() as i32)?;
        }
        for &item in value {
            self.write_u32(item)?;
        }
        Ok(())
    }

    fn write_float_array(&mut self, value: &[f32], write_length: bool) -> Result<(), io::Error> {
        if write_length {
            self.write_i32(value.len() as i32)?;
        }
        for &item in value {
            self.write_f32(item)?;
        }
        Ok(())
    }

    fn write_string_array(&mut self, value: &[String]) -> Result<(), io::Error> {
        self.write_i32(value.len() as i32)?;
        for item in value {
            self.write_aligned_string(item)?;
        }
        Ok(())
    }

    fn write_vector2_array(&mut self, value: &[Vector2]) -> Result<(), io::Error> {
        self.write_i32(value.len() as i32)?;
        for item in value {
            self.write_vector2(item)?; // Note: no & because write_vector2 takes &Vector2
        }
        Ok(())
    }

    fn write_vector4_array(&mut self, value: &[Vector4]) -> Result<(), io::Error> {
        self.write_i32(value.len() as i32)?;
        for item in value {
            self.write_vector4(item)?; // Note: no & because write_vector4 takes &Vector4
        }
        Ok(())
    }

    fn write_matrix_array(&mut self, value: &[Matrix4x4]) -> Result<(), io::Error> {
        self.write_i32(value.len() as i32)?;
        for item in value {
            self.write_matrix4x4(item)?;
        }
        Ok(())
    }
}

/// Binary writer that operates on an in-memory buffer
///
/// This writer accumulates written data in a `Vec<u8>` buffer and supports
/// both sequential and random-access writes. The buffer grows automatically
/// as data is written.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use unity_rs::streams::endian_writer::{BinaryWriter, EndianBinaryWriter, Endian};
///
/// let mut writer = EndianBinaryWriter::new(Endian::Little);
///
/// // Write various types
/// writer.write_i32(42).unwrap();
/// writer.write_f32(3.14159).unwrap();
/// writer.write_bool(true).unwrap();
///
/// // Get the written bytes
/// let bytes = writer.to_bytes();
/// ```
///
/// ## Writing Unity Types
///
/// ```
/// use unity_rs::streams::endian_writer::{BinaryWriter, EndianBinaryWriter, Endian};
/// use unity_rs::math::{Vector2, Vector3};
///
/// let mut writer = EndianBinaryWriter::new(Endian::Little);
///
/// writer.write_vector2(&Vector2::new(1.0, 2.0)).unwrap();
/// writer.write_vector3(&Vector3::new(3.0, 4.0, 5.0)).unwrap();
///
/// let bytes = writer.to_bytes();
/// ```
///
/// ## Round-trip with Reader
///
/// ```
/// use unity_rs::streams::endian_writer::{BinaryWriter, EndianBinaryWriter, Endian};
/// use unity_rs::streams::endian_reader::{BinaryReader, MemoryReader};
///
/// // Write data
/// let mut writer = EndianBinaryWriter::new(Endian::Little);
/// writer.write_i32(100).unwrap();
/// writer.write_f64(2.71828).unwrap();
///
/// // Read it back
/// let bytes = writer.to_bytes();
/// let mut reader = MemoryReader::new(bytes, Endian::Little, 0);
///
/// assert_eq!(reader.read_i32().unwrap(), 100);
/// assert_eq!(reader.read_f64().unwrap(), 2.71828);
/// ```
#[derive(Debug)]
pub struct EndianBinaryWriter {
    data: Vec<u8>,
    position: usize,
    endian: Endian,
    pub flags: u32,
    pub name: String,
}

impl EndianBinaryWriter {
    /// Creates a new EndianBinaryWriter with the specified byte order
    ///
    /// # Arguments
    ///
    /// * `endian` - The byte order for writing multi-byte values
    ///
    /// # Examples
    ///
    /// ```
    /// use unity_rs::streams::endian_writer::{EndianBinaryWriter, Endian};
    ///
    /// let writer_le = EndianBinaryWriter::new(Endian::Little);
    /// let writer_be = EndianBinaryWriter::new(Endian::Big);
    /// ```
    pub fn new(endian: Endian) -> Self {
        EndianBinaryWriter {
            data: Vec::new(),
            position: 0,
            endian,
            flags: 0,
            name: String::new(),
        }
    }

    /// Consumes the writer and returns the written bytes
    ///
    /// This method takes ownership of the writer and returns the underlying
    /// byte buffer containing all written data.
    ///
    /// # Examples
    ///
    /// ```
    /// use unity_rs::streams::endian_writer::{BinaryWriter, EndianBinaryWriter, Endian};
    ///
    /// let mut writer = EndianBinaryWriter::new(Endian::Little);
    /// writer.write_i32(42).unwrap();
    /// writer.write_f32(3.14).unwrap();
    ///
    /// let bytes = writer.to_bytes();
    /// assert_eq!(bytes.len(), 8); // 4 bytes + 4 bytes
    /// ```
    pub fn to_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Returns a reference to the written bytes without consuming the writer
    ///
    /// # Examples
    ///
    /// ```
    /// use unity_rs::streams::endian_writer::{BinaryWriter, EndianBinaryWriter, Endian};
    ///
    /// let mut writer = EndianBinaryWriter::new(Endian::Little);
    /// writer.write_i32(42).unwrap();
    ///
    /// let bytes = writer.bytes(); // Borrow, don't consume
    /// assert_eq!(bytes.len(), 4);
    /// // Can still use writer after this
    /// ```
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}

impl BinaryWriter for EndianBinaryWriter {
    // Core write operation
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        let end_pos = self.position + data.len();

        // Grow buffer if needed
        if end_pos > self.data.len() {
            self.data.resize(end_pos, 0);
        }

        // Write data at current position
        self.data[self.position..end_pos].copy_from_slice(data);
        self.position = end_pos;

        Ok(data.len())
    }

    // Position management
    fn position(&self) -> usize {
        self.position
    }
    fn set_position(&mut self, pos: usize) {
        self.position = pos
    }

    // Endianness
    fn endian(&self) -> Endian {
        self.endian
    }
    fn set_endian(&mut self, endian: Endian) {
        self.endian = endian
    }

    // Length (total bytes written)
    fn len(&self) -> usize {
        self.data.len()
    }
}
