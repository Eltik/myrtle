use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_option_number<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        String(String),
        Num(usize),
    }

    match Option::<StringOrNum>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StringOrNum::Num(n)) => Ok(Some(n)),
        Some(StringOrNum::String(s)) => s
            .parse::<usize>()
            .map(Some)
            .map_err(|_| D::Error::custom("invalid number")),
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub cursor: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_number")]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub total: usize,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, cursor: Option<String>, total: usize, limit: usize) -> Self {
        let has_more = items.len() == limit;
        Self {
            data: items,
            next_cursor: if has_more { cursor } else { None },
            has_more,
            total,
        }
    }
}
