use std::{
    fmt,
    io::{self, BufReader, Read},
    path::Path,
};

use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum DataError {
    Io(std::io::Error),
    Parse {
        table: String,
        error: serde_json::Error,
    },
    Missing {
        table: String,
    },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Parse { table, error } => write!(f, "Failed to parse {table}: {error}"),
            Self::Missing { table } => write!(f, "Missing table: {table}"),
        }
    }
}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::error::Error for DataError {}

pub fn load_table<T: DeserializeOwned>(data_dir: &Path, table_name: &str) -> Result<T, DataError> {
    // Reported here rather than at the 25 call sites; inert outside a boot.
    crate::core::startup::step(table_name);
    let path = data_dir.join(format!("{table_name}.json"));
    let file = std::fs::File::open(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DataError::Missing {
                table: table_name.to_owned(),
            }
        } else {
            DataError::Io(e)
        }
    })?;
    serde_json::from_reader(SanitizingReader::new(BufReader::new(file))).map_err(|e| {
        DataError::Parse {
            table: table_name.to_owned(),
            error: e,
        }
    })
}

/// `Read` adapter that yields the bytes of `sanitize_lone_surrogates(&String::from_utf8_lossy(raw))`
/// without materialising either string. A 1.8 GB table used to cost three full-size buffers
/// (raw, lossy, sanitized) before `serde_json` saw a byte; peak 4.4 GiB for a 354 MiB result.
pub(crate) struct SanitizingReader<R: Read> {
    inner: R,
    chunk: Vec<u8>,
    utf8_tail: Vec<u8>,
    sanitize_tail: Vec<u8>,
    output: Vec<u8>,
    output_pos: usize,
    eof: bool,
}

impl<R: Read> SanitizingReader<R> {
    const CHUNK_SIZE: usize = 64 * 1024;

    pub(crate) fn new(inner: R) -> Self {
        Self::with_chunk_size(inner, Self::CHUNK_SIZE)
    }

    fn with_chunk_size(inner: R, chunk_size: usize) -> Self {
        Self {
            inner,
            chunk: vec![0; chunk_size],
            utf8_tail: Vec::with_capacity(3),
            sanitize_tail: Vec::with_capacity(11),
            output: Vec::with_capacity(chunk_size),
            output_pos: 0,
            eof: false,
        }
    }

    fn trailing_incomplete_utf8_start(bytes: &[u8]) -> Option<usize> {
        let continuation_count = bytes
            .iter()
            .rev()
            .take_while(|&&byte| (byte & 0xC0) == 0x80)
            .count();
        let start = bytes.len().checked_sub(continuation_count + 1)?;
        let expected_len = match bytes[start] {
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => return None,
        };
        let suffix = &bytes[start..];
        let valid_prefix = match suffix {
            [0xE0, second, ..] => *second >= 0xA0,
            [0xED, second, ..] => *second <= 0x9F,
            [0xF0, second, ..] => *second >= 0x90,
            [0xF4, second, ..] => *second <= 0x8F,
            _ => true,
        };
        (valid_prefix && suffix.len() < expected_len).then_some(start)
    }

    fn decode_lossy(&mut self, bytes: &[u8], at_eof: bool) {
        let mut input = std::mem::take(&mut self.utf8_tail);
        input.extend_from_slice(bytes);
        let split_at = if at_eof {
            input.len()
        } else {
            Self::trailing_incomplete_utf8_start(&input).unwrap_or(input.len())
        };
        self.utf8_tail.extend_from_slice(&input[split_at..]);

        for chunk in input[..split_at].utf8_chunks() {
            self.sanitize_tail
                .extend_from_slice(chunk.valid().as_bytes());
            if !chunk.invalid().is_empty() {
                self.sanitize_tail.extend_from_slice("\u{FFFD}".as_bytes());
            }
        }
    }

    fn hex_escape(bytes: &[u8]) -> Option<u16> {
        if bytes.len() >= 6 && bytes[0] == b'\\' && bytes[1] == b'u' {
            std::str::from_utf8(&bytes[2..6])
                .ok()
                .and_then(|hex| u16::from_str_radix(hex, 16).ok())
        } else {
            None
        }
    }

    fn sanitize_available(&mut self) {
        let mut i = 0;
        while i + 12 <= self.sanitize_tail.len() {
            if let Some(cp) = Self::hex_escape(&self.sanitize_tail[i..]) {
                if (0xD800..=0xDBFF).contains(&cp) {
                    if Self::hex_escape(&self.sanitize_tail[i + 6..])
                        .is_some_and(|lo| (0xDC00..=0xDFFF).contains(&lo))
                    {
                        let pair = self.sanitize_tail[i..i + 12].to_vec();
                        self.output.extend_from_slice(&pair);
                        i += 12;
                    } else {
                        self.output.extend_from_slice(b"\\uFFFD");
                        i += 6;
                    }
                    continue;
                }
                if (0xDC00..=0xDFFF).contains(&cp) {
                    self.output.extend_from_slice(b"\\uFFFD");
                    i += 6;
                    continue;
                }
            }
            // Copy a whole character, never a prefix of one: `sanitize_tail` is valid
            // UTF-8 (lossy chunks plus U+FFFD) and `finish` decodes the held-back tail
            // as `&str`, so a drain that split a multibyte character would leave a
            // continuation byte at the front and fail that decode at end of file.
            let step = match self.sanitize_tail[i] {
                0xF0..=0xF4 => 4,
                0xE0..=0xEF => 3,
                0xC2..=0xDF => 2,
                _ => 1,
            };
            let end = (i + step).min(self.sanitize_tail.len());
            self.output.extend_from_slice(&self.sanitize_tail[i..end]);
            i = end;
        }
        self.sanitize_tail.drain(..i);
    }

    fn finish(&mut self) {
        self.decode_lossy(&[], true);
        debug_assert!(self.utf8_tail.is_empty());
        let tail = std::str::from_utf8(&self.sanitize_tail).expect("lossy output is valid UTF-8");
        self.output
            .extend_from_slice(sanitize_lone_surrogates(tail).as_bytes());
        self.sanitize_tail.clear();
    }

    fn fill_output(&mut self) -> io::Result<()> {
        while self.output_pos == self.output.len() && !self.eof {
            self.output.clear();
            self.output_pos = 0;
            let read = self.inner.read(&mut self.chunk)?;
            if read == 0 {
                self.eof = true;
                self.finish();
            } else {
                let bytes = self.chunk[..read].to_vec();
                self.decode_lossy(&bytes, false);
                self.sanitize_available();
            }
        }
        Ok(())
    }
}

impl<R: Read> Read for SanitizingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.fill_output()?;
        let available = &self.output[self.output_pos..];
        let count = available.len().min(buf.len());
        buf[..count].copy_from_slice(&available[..count]);
        self.output_pos += count;
        Ok(count)
    }
}

/// Replace unpaired UTF-16 surrogate escapes (`\uD800`-`\uDFFF` without a valid
/// low-surrogate pair) with the Unicode replacement character escape `\uFFFD`.
/// The upstream `FlatBuffer` JSON emitter occasionally writes lone surrogates,
/// which `serde_json` rejects with "invalid unicode code point".
fn sanitize_lone_surrogates(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut copied = 0;
    let mut i = 0;
    while i + 6 <= bytes.len() {
        if bytes[i] == b'\\'
            && bytes[i + 1] == b'u'
            && let Some(cp) = std::str::from_utf8(&bytes[i + 2..i + 6])
                .ok()
                .and_then(|h| u16::from_str_radix(h, 16).ok())
        {
            if (0xD800..=0xDBFF).contains(&cp) {
                let paired = i + 12 <= bytes.len()
                    && bytes[i + 6] == b'\\'
                    && bytes[i + 7] == b'u'
                    && std::str::from_utf8(&bytes[i + 8..i + 12])
                        .ok()
                        .and_then(|h| u16::from_str_radix(h, 16).ok())
                        .is_some_and(|lo| (0xDC00..=0xDFFF).contains(&lo));
                if paired {
                    i += 12;
                    continue;
                }
                out.push_str(&input[copied..i]);
                out.push_str("\\uFFFD");
                i += 6;
                copied = i;
                continue;
            } else if (0xDC00..=0xDFFF).contains(&cp) {
                out.push_str(&input[copied..i]);
                out.push_str("\\uFFFD");
                i += 6;
                copied = i;
                continue;
            }
            i += 6;
            continue;
        }
        i += 1;
    }
    out.push_str(&input[copied..]);
    out
}

pub fn load_table_or_warn<T: DeserializeOwned + Default>(
    data_dir: &Path,
    table_name: &str,
    warnings: &mut Vec<String>,
) -> T {
    match load_table(data_dir, table_name) {
        Ok(t) => t,
        Err(e) => {
            warnings.push(format!("{table_name}: {e}"));
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::{SanitizingReader, sanitize_lone_surrogates};

    fn sanitize_stream(input: &[u8], chunk_size: usize) -> Vec<u8> {
        let mut reader = SanitizingReader::with_chunk_size(input, chunk_size);
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        output
    }

    fn expected(input: &[u8]) -> Vec<u8> {
        sanitize_lone_surrogates(&String::from_utf8_lossy(input)).into_bytes()
    }

    fn assert_parity(input: &[u8]) {
        for chunk_size in [1, 2, 3, 5, 7, 11, 13, 64, input.len()] {
            assert_eq!(sanitize_stream(input, chunk_size), expected(input));
        }
    }

    #[test]
    fn sanitizing_reader_matches_lossy_sanitization() {
        let cases: &[&[u8]] = &[
            br#"{"plain":"ASCII"}"#,
            br#"{"pair":"\uD83D\uDE00"}"#,
            br#"{"high":"\uD83Dx"}"#,
            br#"{"low":"\uDE00"}"#,
            br#"{"truncated_escape":"\uD83D\uDE0"#,
            br#"{"lowercase":"\ud83d\ude00"}"#,
            "{\"utf8\":\"😀\"}".as_bytes(),
            b"{\"invalid\":\"\xff\"}",
            b"{\"truncated_utf8\":\"\xe2\x82",
        ];
        for case in cases {
            assert_parity(case);
        }
    }

    /// The hold-back window is 11 bytes; when the input ends in a run of 4-byte
    /// characters that window starts inside one of them, which is exactly the
    /// case `finish` must decode as `&str`.
    #[test]
    fn sanitizing_reader_holds_back_whole_characters() {
        assert_parity("{\"k\":\"😀😀😀😀😀\"}".as_bytes());
        assert_parity("{\"k\":\"日本語日本語日本語\"}".as_bytes());
        assert_parity("😀😀😀😀😀".as_bytes());
    }

    #[test]
    fn sanitizing_reader_matches_mixed_input() {
        let mut input = Vec::with_capacity(3 * 1024);
        let mut state = 0x1234_5678_u32;
        let fragments: &[&[u8]] = &[
            br#"{"ascii":"text","#,
            br#""pair":"\uD83D\uDE00","#,
            br#""high":"\uD83Dx","#,
            br#""low":"\uDE00","#,
            "\"utf8\":\"😀\",".as_bytes(),
            b"\"invalid\":\"\xff\",",
            b"\"truncated_like\":\"\xe2\x82",
        ];
        while input.len() < 3 * 1024 {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            input.extend_from_slice(fragments[(state as usize) % fragments.len()]);
        }
        input.truncate(3 * 1024);
        assert_parity(&input);
    }
}
