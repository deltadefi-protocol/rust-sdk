use serde::Serialize;

use crate::OrderRecordStatus;

// Add this struct with your other models
#[derive(Debug, Serialize)]
pub struct OrderRecordParams {
    pub status: OrderRecordStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl OrderRecordParams {
    pub fn new(status: OrderRecordStatus) -> Self {
        Self {
            status,
            limit: None,
            page: None,
            symbol: None,
        }
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_symbol(mut self, symbol: String) -> Self {
        self.symbol = Some(symbol);
        self
    }
}
