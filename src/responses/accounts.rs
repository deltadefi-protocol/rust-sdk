//! Account Response Types
//!
//! This module contains response types for account-related API operations.

use crate::model::{
    AssetBalance, DepositRecord, Order, PaginatedResponse, TransferalRecord, WithdrawalRecord,
};
use serde::{Deserialize, Serialize};

/// Represents the response for creating a new API key.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateNewAPIKeyResponse {
    pub api_key: String,
    pub created_at: String,
}

/// Represents the response for getting the API key.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetAPIKeyResponse {
    pub api_key: String,
    pub created_at: String,
}

/// Represents the response for getting the operation key.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetOperationKeyResponse {
    pub encrypted_operation_key: String,
    pub operation_key_hash: String,
}

/// Represents the response for building a deposit transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildDepositTransactionResponse {
    pub tx_hex: String,
}

/// Represents the response for submitting a deposit transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitDepositTransactionResponse {
    pub tx_hash: String,
}

/// Represents the response for getting deposit records.
pub type GetDepositRecordsResponse = Vec<DepositRecord>;

/// Represents the response for getting withdrawal records.
pub type GetWithdrawalRecordsResponse = Vec<WithdrawalRecord>;

/// Represents the response for getting transferal records.
pub type GetTransferalRecordsResponse = Vec<TransferalRecord>;

/// Represents the response for getting a single order.
/// The API returns the Order directly (not wrapped).
pub type GetOrderResponse = Order;

/// Represents the response for getting open orders (paginated).
/// The API returns a PaginatedResponse with `data` field containing orders.
pub type GetOpenOrdersResponse = PaginatedResponse<Order>;

/// Represents the response for getting trade orders (paginated).
/// The API returns a PaginatedResponse with `data` field containing orders.
pub type GetTradeOrdersResponse = PaginatedResponse<Order>;

/// Represents the response for getting account trades (paginated).
/// The API returns a PaginatedResponse with `data` field containing orders.
pub type GetAccountTradesResponse = PaginatedResponse<Order>;

/// Represents the response for building a withdrawal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildWithdrawalTransactionResponse {
    pub tx_hex: String,
}

/// Represents the response for building a transferal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildTransferalTransactionResponse {
    pub tx_hex: String,
}

/// Represents the response for building a request transferal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildRequestTransferalTransactionResponse {
    pub tx_hex: String,
}

/// Represents the response for submitting a withdrawal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitWithdrawalTransactionResponse {
    pub tx_hash: String,
}

/// Represents the response for submitting a transferal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitTransferalTransactionResponse {
    pub tx_hash: String,
}

/// Represents the response for submitting a request transferal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitRequestTransferalTransactionResponse {
    pub tx_hash: String,
}

/// Represents the response for getting account information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetAccountInfoResponse {
    pub api_key: String,
    pub api_limit: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents the response for getting account balances.
pub type GetAccountBalanceResponse = Vec<AssetBalance>;

/// Represents the response for getting max deposit.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMaxDepositResponse {
    pub max_deposit: String,
}

/// Represents the response for getting spot account.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetSpotAccountResponse {
    pub account_id: String,
    pub account_type: String,
    pub encrypted_operation_key: String,
    pub operation_key_hash: String,
    pub created_at: String,
}

/// Represents the response for creating spot account.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSpotAccountResponse {
    pub account_id: String,
    pub account_type: String,
    pub encrypted_operation_key: String,
    pub operation_key_hash: String,
    pub created_at: String,
}

/// Represents the response for updating spot account.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSpotAccountResponse {
    pub account_id: String,
    pub account_type: String,
    pub encrypted_operation_key: String,
    pub operation_key_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents the response for getting transferal record by tx hash.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTransferalRecordByTxHashResponse {
    #[serde(flatten)]
    pub record: TransferalRecord,
}
