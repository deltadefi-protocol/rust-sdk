//! Account Management Module
//!
//! This module provides functionality for managing DeltaDeFi accounts, including:
//! - Operation key management
//! - Deposit and withdrawal operations
//! - Balance inquiries
//! - Transaction history
//! - Order queries
//! - Spot account management

use serde::Serialize;
use serde_json::from_str;
use whisky::{Asset, UTxO, WError};

use super::Api;
use crate::requests::{BuildTransferalTransactionRequest, BuildWithdrawalTransactionRequest};
use crate::responses::accounts::*;

/// Query parameters for paginated order requests.
#[derive(Debug, Serialize)]
pub struct OrderQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}

/// Client for account-related operations on the DeltaDeFi platform.
///
/// Provides methods for managing account balances, deposits, withdrawals,
/// and accessing transaction history. All operations require proper API
/// authentication and may require operation key signing for transactions.
pub struct Accounts {
    /// Internal API client
    pub api: Api,
    /// Base path for account endpoints
    pub path_url: String,
}

impl Accounts {
    pub fn new(api: Api) -> Self {
        Accounts {
            api,
            path_url: "/accounts".to_string(),
        }
    }

    // ==================== API Key & Operation Key ====================

    /// Retrieves the encrypted operation key from the DeltaDeFi API.
    pub async fn get_operation_key(&self) -> Result<GetOperationKeyResponse, WError> {
        let url = format!("{}/operation-key", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_operation_key"))?)
    }

    /// Creates a new API key for the account.
    pub async fn create_new_api_key(&self) -> Result<CreateNewAPIKeyResponse, WError> {
        let url = format!("{}/new-api-key", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("create_new_api_key"))?)
    }

    /// Gets the current API key information.
    pub async fn get_api_key(&self) -> Result<GetAPIKeyResponse, WError> {
        let url = format!("{}/api-key", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_api_key"))?)
    }

    // ==================== Spot Account ====================

    /// Gets the spot account information.
    pub async fn get_spot_account(&self) -> Result<GetSpotAccountResponse, WError> {
        let url = format!("{}/spot-account", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_spot_account"))?)
    }

    /// Creates a new spot account.
    ///
    /// # Arguments
    ///
    /// * `encrypted_operation_key` - The encrypted operation key for the account
    /// * `operation_key_hash` - The hash of the operation key
    pub async fn create_spot_account(
        &self,
        encrypted_operation_key: &str,
        operation_key_hash: &str,
    ) -> Result<CreateSpotAccountResponse, WError> {
        let url = format!("{}/spot-account", self.path_url);
        let payload = serde_json::json!({
            "encrypted_operation_key": encrypted_operation_key,
            "operation_key_hash": operation_key_hash,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("create_spot_account"))?)
    }

    /// Updates the spot account.
    ///
    /// # Arguments
    ///
    /// * `encrypted_operation_key` - The new encrypted operation key
    /// * `operation_key_hash` - The new operation key hash
    pub async fn update_spot_account(
        &self,
        encrypted_operation_key: &str,
        operation_key_hash: &str,
    ) -> Result<UpdateSpotAccountResponse, WError> {
        let url = format!("{}/spot-account", self.path_url);
        let payload = serde_json::json!({
            "encrypted_operation_key": encrypted_operation_key,
            "operation_key_hash": operation_key_hash,
        });
        let response = self.api.patch(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("update_spot_account"))?)
    }

    // ==================== Balance ====================

    /// Retrieves the current account balance for all assets.
    pub async fn get_account_balance(&self) -> Result<GetAccountBalanceResponse, WError> {
        let url = format!("{}/balance", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_account_balance"))?)
    }

    // ==================== Deposit ====================

    /// Gets the maximum deposit amount allowed.
    pub async fn get_max_deposit(&self) -> Result<GetMaxDepositResponse, WError> {
        let url = format!("{}/max-deposit", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_max_deposit"))?)
    }

    /// Builds a deposit transaction for transferring assets to the DeltaDeFi protocol.
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

    /// Submits a signed deposit transaction.
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

    /// Gets deposit records.
    pub async fn get_deposit_records(&self) -> Result<GetDepositRecordsResponse, WError> {
        let url = format!("{}/deposit-records", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_deposit_records"))?)
    }

    // ==================== Withdrawal ====================

    /// Builds a withdrawal transaction.
    ///
    /// # Arguments
    ///
    /// * `request` - Withdrawal parameters. For a standard withdrawal, build with
    ///   [`BuildWithdrawalTransactionRequest::new`]. For an owner-only vault
    ///   withdrawal, use [`BuildWithdrawalTransactionRequest::new_vault`] and
    ///   optionally chain [`with_from_account_id`](BuildWithdrawalTransactionRequest::with_from_account_id).
    pub async fn build_withdrawal_transaction(
        &self,
        request: BuildWithdrawalTransactionRequest,
    ) -> Result<BuildWithdrawalTransactionResponse, WError> {
        let url = format!("{}/withdrawal/build", self.path_url);
        let response = self.api.post(&url, request).await?;
        Ok(from_str(&response).map_err(WError::from_err("build_withdrawal_transaction"))?)
    }

    /// Submits a signed withdrawal transaction.
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

    /// Gets withdrawal records.
    pub async fn get_withdrawal_records(&self) -> Result<GetWithdrawalRecordsResponse, WError> {
        let url = format!("{}/withdrawal-records", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_withdrawal_records"))?)
    }

    // ==================== Transferal ====================

    /// Builds a transferal transaction.
    ///
    /// # Arguments
    ///
    /// * `request` - Transferal parameters. Either `to_address` or `to_account_id`
    ///   must be set; use [`BuildTransferalTransactionRequest::to_address`] or
    ///   [`BuildTransferalTransactionRequest::to_account_id`] respectively. Chain
    ///   `with_transferal_type`, `with_vault_id`, `with_from_account_id` as needed.
    pub async fn build_transferal_transaction(
        &self,
        request: BuildTransferalTransactionRequest,
    ) -> Result<BuildTransferalTransactionResponse, WError> {
        let url = format!("{}/transferal/build", self.path_url);
        let response = self.api.post(&url, request).await?;
        Ok(from_str(&response).map_err(WError::from_err("build_transferal_transaction"))?)
    }

    /// Submits a signed transferal transaction.
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

    /// Builds a request transferal transaction.
    pub async fn build_request_transferal_transaction(
        &self,
        transferal_amount: Vec<Asset>,
        to_address: &str,
    ) -> Result<BuildRequestTransferalTransactionResponse, WError> {
        let url = format!("{}/request-transferal/build", self.path_url);
        let payload = serde_json::json!({
            "transferal_amount": transferal_amount,
            "to_address": to_address,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("build_request_transferal_transaction"))?)
    }

    /// Submits a signed request transferal transaction.
    pub async fn submit_request_transferal_transaction(
        &self,
        signed_tx: &str,
    ) -> Result<SubmitRequestTransferalTransactionResponse, WError> {
        let url = format!("{}/request-transferal/submit", self.path_url);
        let payload = serde_json::json!({
            "signed_tx": signed_tx,
        });
        let response = self.api.post(&url, payload).await?;
        Ok(from_str(&response).map_err(WError::from_err("submit_request_transferal_transaction"))?)
    }

    /// Gets transferal records.
    pub async fn get_transferal_records(&self) -> Result<GetTransferalRecordsResponse, WError> {
        let url = format!("{}/transferal-records", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_transferal_records"))?)
    }

    /// Gets a specific transferal record by transaction hash.
    pub async fn get_transferal_record_by_tx_hash(
        &self,
        tx_hash: &str,
    ) -> Result<GetTransferalRecordByTxHashResponse, WError> {
        let url = format!("{}/transferal-records/{}", self.path_url, tx_hash);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_transferal_record_by_tx_hash"))?)
    }

    // ==================== Orders ====================

    /// Gets a single order by ID.
    pub async fn get_order(&self, order_id: &str) -> Result<GetOrderResponse, WError> {
        let url = format!("{}/order/{}", self.path_url, order_id);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_order"))?)
    }

    /// Gets open orders with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Optional trading pair symbol to filter by
    /// * `limit` - Number of records per page (default: 10)
    /// * `page` - Page number (default: 1)
    pub async fn get_open_orders(
        &self,
        symbol: Option<String>,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> Result<GetOpenOrdersResponse, WError> {
        let url = format!("{}/open-orders", self.path_url);
        let params = OrderQueryParams {
            symbol,
            limit: Some(limit.unwrap_or(10)),
            page: Some(page.unwrap_or(1)),
        };
        let response = self.api.get_with_params(&url, &params).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_open_orders"))?)
    }

    /// Gets trade orders (order history) with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Optional trading pair symbol to filter by
    /// * `limit` - Number of records per page (default: 10)
    /// * `page` - Page number (default: 1)
    pub async fn get_trade_orders(
        &self,
        symbol: Option<String>,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> Result<GetTradeOrdersResponse, WError> {
        let url = format!("{}/trade-orders", self.path_url);
        let params = OrderQueryParams {
            symbol,
            limit: Some(limit.unwrap_or(10)),
            page: Some(page.unwrap_or(1)),
        };
        let response = self.api.get_with_params(&url, &params).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_trade_orders"))?)
    }

    /// Gets account trades with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Optional trading pair symbol to filter by
    /// * `limit` - Number of records per page (default: 10)
    /// * `page` - Page number (default: 1)
    pub async fn get_account_trades(
        &self,
        symbol: Option<String>,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> Result<GetAccountTradesResponse, WError> {
        let url = format!("{}/trades", self.path_url);
        let params = OrderQueryParams {
            symbol,
            limit: Some(limit.unwrap_or(10)),
            page: Some(page.unwrap_or(1)),
        };
        let response = self.api.get_with_params(&url, &params).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_account_trades"))?)
    }
}
