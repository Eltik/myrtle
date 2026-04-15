use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default, deserialize_with = "de_opt_u32")]
    pub limit: Option<u32>,
    #[serde(default, deserialize_with = "de_opt_u32")]
    pub offset: Option<u32>,
}

impl Pagination {
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20).min(100)
    }

    pub fn offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

fn de_opt_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrU32<'a> {
        Num(u32),
        Str(&'a str),
    }

    match Option::<StrOrU32>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StrOrU32::Num(n)) => Ok(Some(n)),
        Some(StrOrU32::Str("")) => Ok(None),
        Some(StrOrU32::Str(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
    }
}
