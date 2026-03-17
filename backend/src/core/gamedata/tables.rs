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
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).map_err(|e| DataError::Parse {
        table: table_name.to_owned(),
        error: e,
    })
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
