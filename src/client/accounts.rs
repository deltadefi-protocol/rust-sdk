use serde_json::from_str;
use whisky::{Asset, UTxO, WError};

use super::Api;
use crate::responses::accounts::*;

pub struct Accounts {
    pub api: Api,
    pub path_url: String,
}

impl Accounts {
    pub fn new(api: Api) -> Self {
        Accounts {
            api,
            path_url: "/accounts".to_string(),
        }
    }

    pub async fn get_operation_key(&self) -> Result<GetOperationKeyResponse, WError> {
        let url = format!("{}/operation-key", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_operation_key"))?)
    }

    pub async fn create_new_api_key(&self) -> Result<CreateNewAPIKeyResponse, WError> {
        let url = format!("{}/new-api-key", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("create_new_api_key"))?)
    }

    pub async fn get_deposit_records(&self) -> Result<GetDepositRecordsResponse, WError> {
        let url = format!("{}/deposit-records", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_deposit_records"))?)
    }

    pub async fn get_withdrawal_records(&self) -> Result<GetWithdrawalRecordsResponse, WError> {
        let url = format!("{}/withdrawal-records", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_withdrawal_records"))?)
    }

    pub async fn get_order_records(&self) -> Result<GetOrderRecordResponse, WError> {
        let url = format!("{}/order-records", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_order_records"))?)
    }

    pub async fn get_account_balance(&self) -> Result<GetAccountBalanceResponse, WError> {
        let url = format!("{}/balance", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_account_balance"))?)
    }

    pub async fn build_deposit_transaction(
        &self,
        deposit_amount: Vec<Asset>,
        input_utxos: Vec<UTxO>,
    ) -> Result<BuildDepositTransactionResponse, WError> {
        let url = format!("{}/deposit/build", self.path_url);
        let payload = serde_json::json!({
            "deposit_amount": deposit_amount,
            "input_utxos": input_utxos,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("build_deposit_transaction"))?)
    }

    pub async fn build_withdrawal_transaction(
        &self,
        withdrawal_amount: Vec<Asset>,
    ) -> Result<BuildWithdrawalTransactionResponse, WError> {
        let url = format!("{}/withdrawal/build", self.path_url);
        let payload = serde_json::json!({
            "withdrawal_amount": withdrawal_amount,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("build_withdrawal_transaction"))?)
    }

    pub async fn build_transferal_transaction(
        &self,
        transferal_amount: Vec<Asset>,
        to_address: &str,
    ) -> Result<BuildTransferalTransactionResponse, WError> {
        let url = format!("{}/transferal/build", self.path_url);
        let payload = serde_json::json!({
            "transferal_amount": transferal_amount,
            "to_address": to_address,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("build_transferal_transaction"))?)
    }

    pub async fn submit_deposit_transaction(
        &self,
        signed_tx: &str,
    ) -> Result<SubmitDepositTransactionResponse, WError> {
        let url = format!("{}/deposit/submit", self.path_url);
        let payload = serde_json::json!({
            "signed_tx": signed_tx,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("submit_deposit_transaction"))?)
    }

    pub async fn submit_withdrawal_transaction(
        &self,
        signed_tx: &str,
    ) -> Result<SubmitWithdrawalTransactionResponse, WError> {
        let url = format!("{}/withdrawal/submit", self.path_url);
        let payload = serde_json::json!({
            "signed_tx": signed_tx,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("submit_withdrawal_transaction"))?)
    }

    pub async fn submit_transferal_transaction(
        &self,
        signed_tx: &str,
    ) -> Result<SubmitTransferalTransactionResponse, WError> {
        let url = format!("{}/transferal/submit", self.path_url);
        let payload = serde_json::json!({
            "signed_tx": signed_tx,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("submit_transferal_transaction"))?)
    }
}
