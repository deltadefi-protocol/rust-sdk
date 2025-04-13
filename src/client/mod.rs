mod accounts;
mod app;
mod market;
mod order;

use accounts::Accounts;
use app::App;
use market::Market;
use order::Order;

use reqwest::RequestBuilder;
use serde::Serialize;
use whisky::{Bip32KeyGenerator, Network, WError};

pub struct DeltaDeFi {
    pub accounts: Accounts,
    pub app: App,
    pub market: Market,
    pub order: Order,
    pub wallet: Option<Bip32KeyGenerator>,
}

impl DeltaDeFi {
    pub fn new(api_key: String, network: Network, signing_key: Option<String>) -> Self {
        let wallet = match signing_key {
            Some(key) => Some(Bip32KeyGenerator::new(&key)),
            None => None,
        };

        let api = Api::new(api_key, network);

        DeltaDeFi {
            accounts: Accounts::new(api.clone()),
            app: App::new(api.clone()),
            market: Market::new(api.clone()),
            order: Order::new(api),
            wallet,
        }
    }
}

#[derive(Clone)]
pub struct Api {
    pub base_url: String,
    pub api_key: String,
    pub network: Network,
    pub http_client: reqwest::Client,
}

impl Api {
    pub fn new(api_key: String, network: Network) -> Self {
        let base_url = match network {
            Network::Mainnet => "https://api-staging.deltadefi.io".to_string(),
            _ => "https://api-staging.deltadefi.io".to_string(),
        };

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Api {
            api_key,
            network,
            base_url,
            http_client,
        }
    }

    async fn send_request(
        &self,
        req: RequestBuilder,
        response_body: &mut String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let req = req
            .header("Accept", "application/json")
            .header("X-API-KEY", &self.api_key)
            .build()?;

        let response = self.http_client.execute(req).await?;

        if response.status().is_success() {
            *response_body = response.text().await?;
            Ok(())
        } else {
            Err(format!("Error: {}", response.status()).into())
        }
    }

    pub async fn get(&self, url: &str) -> Result<String, WError> {
        let req = self.http_client.get(format!("{}{}", &self.base_url, url));
        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(WError::from_err("DeltaDeFi - get - send_request"))?;
        Ok(response_body)
    }

    pub async fn post<T: Serialize>(&self, url: &str, body: T) -> Result<String, WError> {
        let json_body = serde_json::to_string(&body)
            .map_err(WError::from_err("DeltaDeFi - post - json_body"))?;

        let req = self
            .http_client
            .post(format!("{}{}", &self.base_url, url))
            .header("Content-Type", "application/json")
            .body(json_body);

        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(WError::from_err("Blockfrost - post - send_request"))?;
        Ok(response_body)
    }

    pub async fn delete<T: Serialize>(&self, url: &str, body: T) -> Result<String, WError> {
        let json_body = serde_json::to_string(&body)
            .map_err(WError::from_err("DeltaDeFi - post - json_body"))?;

        let response = self
            .http_client
            .delete(url)
            .body(json_body)
            .send()
            .await
            .map_err(WError::from_err("DeltaDeFi - delete - send"))?;
        let body = response
            .text()
            .await
            .map_err(WError::from_err("DeltaDeFi - delete - text"))?;
        Ok(body)
    }
}
