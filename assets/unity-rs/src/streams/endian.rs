/// Byte order for reading and writing binary data
///
/// # Examples
///
/// ```
/// use unity_rs::streams::endian::Endian;
///
/// let little = Endian::Little;  // Intel/AMD x86
/// let big = Endian::Big;        // Network byte order
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endian {
    /// Little-endian byte order (least significant byte first)
    Little,
    /// Big-endian byte order (most significant byte first)
    Big,
}
