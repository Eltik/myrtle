use std::{fmt, path::Path};

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
            DataError::Io(e) => write!(f, "IO error: {e}"),
            DataError::Parse { table, error } => write!(f, "Failed to parse {table}: {error}"),
            DataError::Missing { table } => write!(f, "Missing table: {table}"),
        }
    }
}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e)
    }
}

impl std::error::Error for DataError {}

pub fn load_table<T: DeserializeOwned>(data_dir: &Path, table_name: &str) -> Result<T, DataError> {
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
    let mut raw = Vec::new();
    std::io::Read::read_to_end(&mut std::io::BufReader::new(file), &mut raw)?;
    let contents = String::from_utf8_lossy(&raw);
    let sanitized = sanitize_lone_surrogates(&contents);
    serde_json::from_str(&sanitized).map_err(|e| DataError::Parse {
        table: table_name.to_owned(),
        error: e,
    })
}

/// Replace unpaired UTF-16 surrogate escapes (`\uD800`-`\uDFFF` without a valid
/// low-surrogate pair) with the Unicode replacement character escape `\uFFFD`.
/// The upstream FlatBuffer JSON emitter occasionally writes lone surrogates,
/// which serde_json rejects with "invalid unicode code point".
fn sanitize_lone_surrogates(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut copied = 0;
    let mut i = 0;
    while i + 6 <= bytes.len() {
        if bytes[i] == b'\\' && bytes[i + 1] == b'u' {
            if let Some(cp) = std::str::from_utf8(&bytes[i + 2..i + 6])
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
