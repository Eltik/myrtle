use crate::math::{Color, Matrix4x4, Quaternion, Rectangle, Vector2, Vector3, Vector4};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use half::f16;
use std::io::{self, Cursor};
use std::io::{Read, Seek, SeekFrom};

pub use crate::streams::endian::Endian;

/// Trait for reading binary data with endianness support
///
/// Provides methods for reading primitive types, strings, arrays, and Unity-specific
/// types from binary data. All methods handle endianness conversion automatically.
///
/// # Required Methods
///
/// Implementations must provide:
/// - `read()` - Core byte reading
/// - `position()`, `set_position()` - Position tracking
/// - `endian()`, `set_endian()` - Endianness control
/// - `bytes()` - Access to raw data (may be empty for streams)
/// - `len()` - Total data length
/// - `base_offset()` - Starting offset in the underlying data
///
/// # Default Methods
///
/// The trait provides default implementations for all reading methods
/// (primitives, strings, arrays, Unity types) that build on the required methods.
///
/// # Examples
///
/// ```
/// use unity_rs::streams::endian_reader::{BinaryReader, MemoryReader, Endian};
///
/// let data = vec![1, 0, 0, 0, 2, 0, 0, 0];
/// let mut reader = MemoryReader::new(data, Endian::Little, 0);
///
/// let value1 = reader.read_i32().unwrap();  // 1
/// let value2 = reader.read_i32().unwrap();  // 2
/// ```
pub trait BinaryReader {
    // Required Methods

    /// Reads exactly `size` bytes from the current position
    ///
    /// # Errors
    ///
    /// Returns an error if there aren't enough bytes to read
    fn read(&mut self, size: usize) -> Result<Vec<u8>, io::Error>;

    /// Returns the current read position (relative to `base_offset`)
    fn position(&self) -> usize;

    /// Sets the current read position (relative to `base_offset`)
    fn set_position(&mut self, pos: usize);

    /// Returns the current endianness
    fn endian(&self) -> Endian;

    /// Sets the endianness for subsequent reads
    fn set_endian(&mut self, endian: Endian);

    /// Returns a reference to all bytes (may be empty for stream-based readers)
    fn bytes(&self) -> &[u8];

    /// Returns the total length of the data
    fn len(&self) -> usize;

    /// Returns the base offset in the underlying data source
    fn base_offset(&self) -> usize;

    // Default Implementations
    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, io::Error> {
        self.read(count)
    }

    fn read_u8(&mut self) -> Result<u8, io::Error> {
        let bytes = self.read(1)?;
        Ok(bytes[0])
    }

    /// Reads a signed 8-bit integer
    ///
    /// # Examples
    ///
    /// ```
    /// # use unity_rs::streams::endian_reader::{BinaryReader, MemoryReader, Endian};
    /// let data = vec![0xFF];
    /// let mut reader = MemoryReader::new(data, Endian::Little, 0);
    /// assert_eq!(reader.read_i8().unwrap(), -1);
    /// ```
    fn read_i8(&mut self) -> Result<i8, io::Error> {
        let bytes = self.read(1)?;
        Ok(bytes[0] as i8)
    }

    fn read_u16(&mut self) -> Result<u16, io::Error> {
        let bytes = self.read(2)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_u16::<LittleEndian>(),
            Endian::Big => cursor.read_u16::<BigEndian>(),
        }
    }

    fn read_i16(&mut self) -> Result<i16, io::Error> {
        let bytes = self.read(2)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_i16::<LittleEndian>(),
            Endian::Big => cursor.read_i16::<BigEndian>(),
        }
    }

    fn read_u32(&mut self) -> Result<u32, io::Error> {
        let bytes = self.read(4)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_u32::<LittleEndian>(),
            Endian::Big => cursor.read_u32::<BigEndian>(),
        }
    }

    fn read_i32(&mut self) -> Result<i32, io::Error> {
        let bytes = self.read(4)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_i32::<LittleEndian>(),
            Endian::Big => cursor.read_i32::<BigEndian>(),
        }
    }

    fn read_u64(&mut self) -> Result<u64, io::Error> {
        let bytes = self.read(8)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_u64::<LittleEndian>(),
            Endian::Big => cursor.read_u64::<BigEndian>(),
        }
    }

    fn read_i64(&mut self) -> Result<i64, io::Error> {
        let bytes = self.read(8)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_i64::<LittleEndian>(),
            Endian::Big => cursor.read_i64::<BigEndian>(),
        }
    }

    fn read_f32(&mut self) -> Result<f32, io::Error> {
        let bytes = self.read(4)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_f32::<LittleEndian>(),
            Endian::Big => cursor.read_f32::<BigEndian>(),
        }
    }

    fn read_f64(&mut self) -> Result<f64, io::Error> {
        let bytes = self.read(8)?;
        let mut cursor = Cursor::new(bytes);
        match self.endian() {
            Endian::Little => cursor.read_f64::<LittleEndian>(),
            Endian::Big => cursor.read_f64::<BigEndian>(),
        }
    }

    fn read_bool(&mut self) -> Result<bool, io::Error> {
        let byte = self.read_u8()?;
        Ok(byte != 0)
    }

    fn read_string(&mut self, size: Option<usize>) -> Result<String, io::Error> {
        let bytes = match size {
            Some(n) => self.read(n)?,
            None => {
                let mut result = Vec::new();
                loop {
                    let byte = self.read_u8()?;
                    if byte == 0 {
                        break;
                    }
                    result.push(byte);
                }
                result
            }
        };
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_aligned_string(&mut self) -> Result<String, io::Error> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let length = self.read_i32()?;
        let bytes = self.read(length as usize)?;
        self.align_stream(4);
        // Try UTF-8 first, if that fails encode as base64 with prefix
        match String::from_utf8(bytes.clone()) {
            Ok(s) => Ok(s),
            Err(_) => {
                // Encode binary data as base64 with a prefix to identify it
                Ok(format!("base64:{}", STANDARD.encode(&bytes)))
            }
        }
    }

    fn read_byte_array(&mut self) -> Result<Vec<u8>, io::Error> {
        let length = self.read_i32()?;
        self.read(length as usize)
    }

    fn read_bool_array(&mut self, length: Option<usize>) -> Result<Vec<bool>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_bool()?);
        }
        Ok(result)
    }

    fn align_stream(&mut self, alignment: usize) {
        let pos = self.position();
        let remainder = pos % alignment;
        if remainder != 0 {
            self.set_position(pos + (alignment - remainder));
        }
    }

    // Array methods
    fn read_array<T, F>(&mut self, mut func: F, length: usize) -> Result<Vec<T>, io::Error>
    where
        F: FnMut(&mut Self) -> Result<T, io::Error>,
    {
        let mut result = Vec::with_capacity(length);
        for _ in 0..length {
            result.push(func(self)?);
        }
        Ok(result)
    }

    fn read_i16_array(&mut self, length: Option<usize>) -> Result<Vec<i16>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_i16()?);
        }
        Ok(result)
    }

    fn read_u16_array(&mut self, length: Option<usize>) -> Result<Vec<u16>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_u16()?);
        }
        Ok(result)
    }

    fn read_i32_array(&mut self, length: Option<usize>) -> Result<Vec<i32>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        self.read_array(|reader| reader.read_i32(), len)
    }

    fn read_u32_array(&mut self, length: Option<usize>) -> Result<Vec<u32>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        self.read_array(|reader| reader.read_u32(), len)
    }

    fn read_i64_array(&mut self, length: Option<usize>) -> Result<Vec<i64>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_i64()?);
        }
        Ok(result)
    }

    fn read_u64_array(&mut self, length: Option<usize>) -> Result<Vec<u64>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_u64()?);
        }
        Ok(result)
    }

    fn read_f32_array(&mut self, length: Option<usize>) -> Result<Vec<f32>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        self.read_array(|reader| reader.read_f32(), len)
    }

    fn read_f64_array(&mut self, length: Option<usize>) -> Result<Vec<f64>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_f64()?);
        }
        Ok(result)
    }

    fn read_u8_array(&mut self, length: Option<usize>) -> Result<Vec<u8>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_u8()?);
        }
        Ok(result)
    }

    fn read_string_to_null(&mut self, max_length: usize) -> Result<String, io::Error> {
        let mut bytes = Vec::new();
        for _ in 0..max_length {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // String and 2D array methods
    fn read_string_array(&mut self) -> Result<Vec<String>, io::Error> {
        let length = self.read_i32()? as usize;
        self.read_array(|reader| reader.read_aligned_string(), length)
    }

    fn read_u_int_array_array(
        &mut self,
        length: Option<usize>,
    ) -> Result<Vec<Vec<u32>>, io::Error> {
        let len = match length {
            Some(n) => n,
            None => self.read_i32()? as usize,
        };
        self.read_array(|reader| reader.read_u32_array(None), len)
    }

    // Utility methods
    fn real_offset(&self) -> usize {
        self.base_offset() + self.position()
    }

    fn read_the_rest(&mut self, obj_start: usize, obj_size: usize) -> Result<Vec<u8>, io::Error> {
        let remaining = obj_size.saturating_sub(self.position() - obj_start);
        self.read_bytes(remaining)
    }

    fn read_half(&mut self) -> Result<f32, io::Error> {
        let bytes = self.read(2)?;
        let mut cursor = Cursor::new(bytes);
        let bits = match self.endian() {
            Endian::Little => cursor.read_u16::<LittleEndian>()?,
            Endian::Big => cursor.read_u16::<BigEndian>()?,
        };
        Ok(f16::from_bits(bits).to_f32())
    }

    // Unity math types
    fn read_vector2(&mut self) -> Result<Vector2, io::Error> {
        Ok(Vector2::new(self.read_f32()?, self.read_f32()?))
    }

    fn read_vector3(&mut self) -> Result<Vector3, io::Error> {
        Ok(Vector3::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_vector4(&mut self) -> Result<Vector4, io::Error> {
        Ok(Vector4::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_quaternion(&mut self) -> Result<Quaternion, io::Error> {
        Ok(Quaternion::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_rectangle_f(&mut self) -> Result<Rectangle, io::Error> {
        Ok(Rectangle::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_color_uint(&mut self) -> Result<Color, io::Error> {
        let r = self.read_u8()? as f32 / 255.0;
        let g = self.read_u8()? as f32 / 255.0;
        let b = self.read_u8()? as f32 / 255.0;
        let a = self.read_u8()? as f32 / 255.0;
        Ok(Color::new(r, g, b, a))
    }

    fn read_color4(&mut self) -> Result<Color, io::Error> {
        Ok(Color::new(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_matrix(&mut self) -> Result<Matrix4x4, io::Error> {
        let floats = self.read_f32_array(Some(16))?;
        let mut data = [0.0f32; 16];
        data.copy_from_slice(&floats);
        Ok(Matrix4x4::new(data))
    }

    // Unity type methods
    fn read_vector2_array(&mut self) -> Result<Vec<Vector2>, io::Error> {
        let length = self.read_i32()? as usize;
        self.read_array(|reader| reader.read_vector2(), length)
    }

    fn read_vector4_array(&mut self) -> Result<Vec<Vector4>, io::Error> {
        let length = self.read_i32()? as usize;
        self.read_array(|reader| reader.read_vector4(), length)
    }

    fn read_matrix_array(&mut self) -> Result<Vec<Matrix4x4>, io::Error> {
        let length = self.read_i32()? as usize;
        self.read_array(|reader| reader.read_matrix(), length)
    }
}

/// Binary reader that operates on in-memory data
///
/// This is the most efficient reader type when all data is already in memory.
/// It provides fast random access to the data.
///
/// # Examples
///
/// ```
/// use unity_rs::streams::endian_reader::{BinaryReader, MemoryReader, Endian};
///
/// let data = vec![
///     3, 0, 0, 0,   // length: 3
///     10, 0, 0, 0,  // value: 10
///     20, 0, 0, 0,  // value: 20
///     30, 0, 0, 0,  // value: 30
/// ];
///
/// let mut reader = MemoryReader::new(data, Endian::Little, 0);
/// let array = reader.read_i32_array(None).unwrap();
/// assert_eq!(array, vec![10, 20, 30]);
/// ```
/// Binary reader that operates on in-memory data
///
/// This is the most efficient reader type when all data is already in memory.
/// It provides fast random access to the data.
///
/// # Examples
///
/// ```
/// use unity_rs::streams::endian_reader::{BinaryReader, MemoryReader, Endian};
///
/// let data = vec![
///     3, 0, 0, 0,   // length: 3
///     10, 0, 0, 0,  // value: 10
///     20, 0, 0, 0,  // value: 20
///     30, 0, 0, 0,  // value: 30
/// ];
///
/// let mut reader = MemoryReader::new(data, Endian::Little, 0);
/// let array = reader.read_i32_array(None).unwrap();
/// assert_eq!(array, vec![10, 20, 30]);
/// ```
#[derive(Debug)]
pub struct MemoryReader {
    position: usize,
    data: Vec<u8>,
    endian: Endian,
    offset: usize,
}

impl MemoryReader {
    /// Creates a new MemoryReader
    ///
    /// # Arguments
    ///
    /// * `data` - The byte data to read from
    /// * `endian` - The byte order for reading multi-byte values
    /// * `offset` - Starting offset in the data (usually 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use unity_rs::streams::endian_reader::{MemoryReader, Endian};
    ///
    /// let data = vec![1, 2, 3, 4];
    /// let reader = MemoryReader::new(data, Endian::Little, 0);
    /// ```
    pub fn new(data: Vec<u8>, endian: Endian, offset: usize) -> Self {
        MemoryReader {
            position: offset,
            data,
            endian,
            offset,
        }
    }

    /// Returns the total length of the underlying data buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the underlying data buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl BinaryReader for MemoryReader {
    fn read(&mut self, size: usize) -> Result<Vec<u8>, io::Error> {
        let start = self.position;
        let end = start + size;

        if end > self.data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough data to read",
            ));
        }

        self.position = end;
        Ok(self.data[start..end].to_vec())
    }

    fn position(&self) -> usize {
        self.position
    }

    fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }

    fn endian(&self) -> Endian {
        self.endian
    }

    fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    fn bytes(&self) -> &[u8] {
        &self.data
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn base_offset(&self) -> usize {
        self.offset
    }
}

/// Binary reader that operates on seekable streams (like files)
///
/// This reader can read from any source that implements `Read + Seek`,
/// such as files or in-memory buffers. It's useful when you don't want
/// to load all data into memory at once.
///
/// # Examples
///
/// ```no_run
/// use unity_rs::streams::endian_reader::{BinaryReader, StreamReader, Endian};
/// use std::fs::File;
///
/// let file = File::open("data.bin").unwrap();
/// let mut reader = StreamReader::new(file, Endian::Little, 0).unwrap();
///
/// let value = reader.read_u32().unwrap();
/// println!("Read: {}", value);
/// ```
/// Binary reader that operates on seekable streams (like files)
///
/// This reader can read from any source that implements `Read + Seek`,
/// such as files or in-memory buffers. It's useful when you don't want
/// to load all data into memory at once.
///
/// # Examples
///
/// ```no_run
/// use unity_rs::streams::endian_reader::{BinaryReader, StreamReader, Endian};
/// use std::fs::File;
///
/// let file = File::open("data.bin").unwrap();
/// let mut reader = StreamReader::new(file, Endian::Little, 0).unwrap();
///
/// let value = reader.read_u32().unwrap();
/// println!("Read: {}", value);
/// ```
pub struct StreamReader<R: Read + Seek> {
    stream: R,
    endian: Endian,
    base_offset: usize,
    position: usize,
    length: usize,
}

impl<R: Read + Seek> StreamReader<R> {
    /// Creates a new StreamReader from a seekable stream
    ///
    /// The stream will be seeked to determine its length and position
    /// at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `stream` - Any type implementing `Read + Seek` (e.g., `File`)
    /// * `endian` - The byte order for reading multi-byte values
    /// * `offset` - Starting offset in the stream (usually 0)
    ///
    /// # Errors
    ///
    /// Returns an error if seeking fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use unity_rs::streams::endian_reader::{StreamReader, Endian};
    /// use std::fs::File;
    ///
    /// let file = File::open("test.bin").unwrap();
    /// let reader = StreamReader::new(file, Endian::Big, 0).unwrap();
    /// ```
    pub fn new(mut stream: R, endian: Endian, offset: usize) -> io::Result<Self> {
        let length = stream.seek(SeekFrom::End(0))? as usize - offset;
        stream.seek(SeekFrom::Start(offset as u64))?;

        Ok(StreamReader {
            stream,
            endian,
            base_offset: offset,
            position: 0,
            length,
        })
    }
}

impl<R: Read + Seek> BinaryReader for StreamReader<R> {
    fn read(&mut self, size: usize) -> Result<Vec<u8>, io::Error> {
        let mut buffer = vec![0u8; size];
        self.stream.read_exact(&mut buffer)?;
        self.position += size;
        Ok(buffer)
    }

    fn position(&self) -> usize {
        self.position
    }

    fn set_position(&mut self, pos: usize) {
        self.stream
            .seek(SeekFrom::Start((self.base_offset + pos) as u64))
            .ok();
        self.position = pos;
    }

    fn endian(&self) -> Endian {
        self.endian
    }

    fn set_endian(&mut self, endian: Endian) {
        self.endian = endian
    }

    fn bytes(&self) -> &[u8] {
        // Can't return bytes from a stream - this is a limitation
        // We'll return empty slice
        &[]
    }

    fn len(&self) -> usize {
        self.length
    }

    fn base_offset(&self) -> usize {
        self.base_offset
    }
}
