mod accounts;
mod app;
mod market;
mod order;

use accounts::Accounts;
use app::App;
use market::Market;
use order::Order;

use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use whisky::{decrypt_with_cipher, WError, Wallet, WalletType};

use crate::{order::SubmitPlaceOrderTransactionResponse, OrderSide, OrderType};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Mainnet,
    Staging,
    Dev,
    Custom(String),
}

pub struct DeltaDeFi {
    pub accounts: Accounts,
    pub app: App,
    pub market: Market,
    pub order: Order,
    pub master_wallet: Option<Wallet>,
    pub operation_wallet: Option<Wallet>,
}

impl DeltaDeFi {
    pub fn new(api_key: String, network: Stage, master_key: Option<WalletType>) -> Self {
        let master_wallet = match master_key {
            Some(key) => Some(Wallet::new(key)),
            None => None,
        };

        let api = Api::new(api_key, network);

        DeltaDeFi {
            accounts: Accounts::new(api.clone()),
            app: App::new(api.clone()),
            market: Market::new(api.clone()),
            order: Order::new(api),
            master_wallet,
            operation_wallet: None,
        }
    }

    pub async fn load_operation_key(&mut self, password: &str) -> Result<(), WError> {
        let res = self
            .accounts
            .get_operation_key()
            .await
            .map_err(WError::from_err("DeltaDeFi - load_operation_key"))?;
        let operation_key = decrypt_with_cipher(&res.encrypted_operation_key, password).map_err(
            WError::from_err("DeltaDeFi - load_operation_key - decrypt_with_cipher"),
        )?;
        let operation_wallet = Wallet::new_root_key(&operation_key);
        self.operation_wallet = Some(operation_wallet);
        Ok(())
    }

    pub fn sign_tx_by_master_key(&self, tx: &str) -> Result<String, WError> {
        if let Some(wallet) = &self.master_wallet {
            wallet.sign_tx(tx)
        } else {
            Err(WError::new("DeltaDeFi", "No wallet found"))
        }
    }

    pub fn sign_tx_by_operation_key(&self, tx: &str) -> Result<String, WError> {
        if let Some(wallet) = &self.operation_wallet {
            wallet.sign_tx(tx)
        } else {
            Err(WError::new("DeltaDeFi", "No wallet found"))
        }
    }

    pub async fn post_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
        limit_slippage: Option<bool>,
        max_slippage_basis_point: Option<u64>,
    ) -> Result<SubmitPlaceOrderTransactionResponse, WError> {
        let build_res = self
            .order
            .build_place_order_transaction(
                symbol,
                side,
                order_type,
                quantity,
                price,
                limit_slippage,
                max_slippage_basis_point,
            )
            .await?;
        let signed_tx = self.sign_tx_by_operation_key(&build_res.tx_hex)?;
        let res = self
            .order
            .submit_place_order_transaction(&build_res.order_id, &signed_tx)
            .await;
        res
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<String, WError> {
        let build_res = self.order.build_cancel_order_transaction(order_id).await?;
        let signed_tx = self.sign_tx_by_operation_key(&build_res.tx_hex)?;
        let res = self
            .order
            .submit_cancel_order_transaction(&signed_tx)
            .await?;
        Ok(res.tx_hash)
    }
}

#[derive(Clone)]
pub struct Api {
    pub base_url: String,
    pub api_key: String,
    pub network: Stage,
    pub http_client: reqwest::Client,
}

impl Api {
    pub fn new(api_key: String, network: Stage) -> Self {
        let base_url = match &network {
            Stage::Mainnet => "https://api-staging.deltadefi.io".to_string(),
            Stage::Staging => "https://api-staging.deltadefi.io".to_string(),
            Stage::Dev => "https://api-dev.deltadefi.io".to_string(),
            Stage::Custom(url) => url.to_string(),
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
            .map_err(WError::from_err("DeltaDeFi - post - send_request"))?;
        Ok(response_body)
    }

    pub async fn delete<T: Serialize>(&self, url: &str, body: T) -> Result<String, WError> {
        let json_body = serde_json::to_string(&body)
            .map_err(WError::from_err("DeltaDeFi - post - json_body"))?;

        let req = self
            .http_client
            .delete(format!("{}{}", &self.base_url, url))
            .header("Content-Type", "application/json")
            .body(json_body);

        let mut response_body = String::new();
        self.send_request(req, &mut response_body)
            .await
            .map_err(WError::from_err("DeltaDeFi - post - send_request"))?;
        Ok(response_body)
    }
}
