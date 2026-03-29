use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
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
