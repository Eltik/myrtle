use std::io::{self, Read};
use std::mem::take;
use std::str::from_utf8;

pub struct SanitizingReader<R: Read> {
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

    pub fn new(inner: R) -> Self {
        Self {
            inner,
            chunk: vec![0; Self::CHUNK_SIZE],
            utf8_tail: Vec::with_capacity(3),
            sanitize_tail: Vec::with_capacity(11),
            output: Vec::with_capacity(Self::CHUNK_SIZE),
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
        let mut input = take(&mut self.utf8_tail);
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

    fn sanitize_available(&mut self) {
        let mut i = 0;
        while i + 12 <= self.sanitize_tail.len() {
            if let Some(cp) = hex_escape(&self.sanitize_tail[i..]) {
                if (0xD800..=0xDBFF).contains(&cp) {
                    if hex_escape(&self.sanitize_tail[i + 6..])
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
        debug_assert_eq!(self.utf8_tail, b"");
        let tail = from_utf8(&self.sanitize_tail).expect("lossy output is valid UTF-8");
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

/// Code unit of the `\uXXXX` escape at the start of `bytes`
fn hex_escape(bytes: &[u8]) -> Option<u16> {
    let hex = bytes.strip_prefix(b"\\u")?.get(..4)?;
    u16::from_str_radix(from_utf8(hex).ok()?, 16).ok()
}

/// Replace unpaired surrogate escapes with `\uFFFD`
fn sanitize_lone_surrogates(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut copied = 0;
    let mut i = 0;
    while i + 6 <= bytes.len() {
        if let Some(cp) = hex_escape(&bytes[i..]) {
            if (0xD800..=0xDBFF).contains(&cp) {
                let paired =
                    hex_escape(&bytes[i + 6..]).is_some_and(|lo| (0xDC00..=0xDFFF).contains(&lo));
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
