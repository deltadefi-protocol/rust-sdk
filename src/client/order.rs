use serde::Serialize;
use serde_json::json;
use whisky::WError;

use super::Api;

pub struct Order {
    pub api: Api,
    pub path_url: String,
}

impl Order {
    pub fn new(api: Api) -> Self {
        Order {
            api,
            path_url: "/order".to_string(),
        }
    }

    /// Builds a place order transaction.
    pub async fn build_place_order_transaction<T: Serialize>(
        &self,
        data: T,
    ) -> Result<String, WError> {
        let url = format!("{}/build", self.path_url);
        self.api.post(&url, data).await
    }

    /// Builds a cancel order transaction.
    pub async fn build_cancel_order_transaction(&self, order_id: &str) -> Result<String, WError> {
        let url = format!("{}/{}/build", self.path_url, order_id);
        self.api.delete(&url, json!({})).await
    }

    /// Submits a place order transaction.
    pub async fn submit_place_order_transaction<T: Serialize>(
        &self,
        data: T,
    ) -> Result<String, WError> {
        let url = format!("{}/submit", self.path_url);
        self.api.post(&url, data).await
    }

    /// Submits a cancel order transaction.
    pub async fn submit_cancel_order_transaction<T: Serialize>(
        &self,
        data: T,
    ) -> Result<String, WError> {
        let url = format!("{}/submit", self.path_url);
        self.api.delete(&url, data).await
    }
}
