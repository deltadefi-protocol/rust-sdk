//! Account Management Module
//!
//! This module provides functionality for managing DeltaDeFi accounts, including:
//! - Operation key management
//! - Deposit and withdrawal operations
//! - Balance inquiries
//! - Transaction history
//! - Account-related API operations

use serde_json::from_str;
use whisky::{Asset, UTxO, WError};

use super::Api;
use crate::{responses::accounts::*, OrderRecordParams, OrderRecordStatus};

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

    /// Retrieves the encrypted operation key from the DeltaDeFi API.
    ///
    /// The operation key is required for signing transactions and must be decrypted
    /// using the account password before it can be used.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the encrypted operation key and its hash,
    /// or a `WError` if the request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let operation_key_response = client.accounts.get_operation_key().await?;
    /// println!("Encrypted key: {}", operation_key_response.encrypted_operation_key);
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - API authentication fails
    /// - Network request fails
    /// - Account doesn't have an operation key configured
    pub async fn get_operation_key(&self) -> Result<GetOperationKeyResponse, WError> {
        let url = format!("{}/operation-key", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_operation_key"))?)
    }

    /// Creates a new API key for the account.
    ///
    /// Generates a new API key that can be used for authentication with the DeltaDeFi API.
    /// This is useful for creating additional API keys or rotating existing ones.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new API key string, or a `WError` if creation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let new_key_response = client.accounts.create_new_api_key().await?;
    /// println!("New API key: {}", new_key_response.api_key);
    /// ```
    ///
    /// # Security
    ///
    /// Store the returned API key securely and never expose it in logs or client-side code.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - API authentication fails
    /// - Account has reached the maximum number of API keys
    /// - Network request fails
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

    pub async fn get_order_records(
        &self,
        status: OrderRecordStatus,
        limit: Option<u32>,
        page: Option<u32>,
        symbol: Option<String>,
    ) -> Result<GetOrderRecordsResponse, WError> {
        let url = format!("{}/order-records", self.path_url);

        // page default to be 1 if none
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(10);

        let mut params = OrderRecordParams::new(status)
            .with_limit(limit)
            .with_page(page);

        if let Some(symbol) = symbol {
            params = params.with_symbol(symbol);
        }

        let response = self.api.get_with_params(&url, &params).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_order_records"))?)
    }

    pub async fn get_order_record(
        &self,
        order_id: &str,
    ) -> Result<GetOrderRecordByIdResponse, WError> {
        let url = format!("{}/order/{}", self.path_url, order_id);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_order_record"))?)
    }

    /// Retrieves the current account balance for all assets.
    ///
    /// Returns the available and locked balances for all assets in the account.
    /// Locked balances represent funds that are currently tied up in open orders
    /// or pending transactions.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing account balances for all assets, or a `WError` if the request fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let balance_response = client.accounts.get_account_balance().await?;
    /// for balance in balance_response.balances {
    ///     println!("{}: {} free, {} locked", 
    ///              balance.asset, balance.free, balance.locked);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - API authentication fails
    /// - Network request fails
    pub async fn get_account_balance(&self) -> Result<GetAccountBalanceResponse, WError> {
        let url = format!("{}/balance", self.path_url);
        let response = self.api.get(&url).await?;
        Ok(from_str(&response).map_err(WError::from_err("get_account_balance"))?)
    }

    /// Builds a deposit transaction for transferring assets to the DeltaDeFi protocol.
    ///
    /// Creates an unsigned transaction that deposits the specified assets from external
    /// UTXOs into your DeltaDeFi account. The transaction must be signed and submitted
    /// using `submit_deposit_transaction`.
    ///
    /// # Arguments
    ///
    /// * `deposit_amount` - Vector of assets and quantities to deposit
    /// * `input_utxos` - Vector of UTXOs to use as inputs for the transaction
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the unsigned transaction hex, or a `WError` if building fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whisky::{Asset, UTxO};
    /// 
    /// let deposit_assets = vec![Asset {
    ///     asset: "ADA".to_string(),
    ///     asset_unit: "lovelace".to_string(),
    ///     qty: 1000000.0,  // 1 ADA
    /// }];
    /// 
    /// let tx_response = client.accounts.build_deposit_transaction(
    ///     deposit_assets,
    ///     input_utxos
    /// ).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - Insufficient funds in input UTXOs
    /// - Invalid asset specifications
    /// - Network request fails
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
