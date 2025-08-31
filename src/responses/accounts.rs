use crate::model::{AssetBalance, DepositRecord, OrderJSON, OrderFillingRecordJSON, WithdrawalRecord};
use serde::{Deserialize, Serialize};

/// Represents the response for creating a new API key.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateNewAPIKeyResponse {
    pub api_key: String,
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

/// Represents a single data item in the order records response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderRecordsData {
    pub orders: Vec<OrderJSON>,
    pub order_filling_records: Vec<OrderFillingRecordJSON>,
}

/// Represents the response for getting order records (paginated).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetOrderRecordsResponse {
    pub data: Vec<OrderRecordsData>,
    pub total_count: i32,
    pub total_page: i32,
}

/// Represents the response for getting a single order record by ID.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetOrderRecordByIdResponse {
    pub order_json: OrderJSON,
}

/// Represents the response for building a withdrawal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildWithdrawalTransactionResponse {
    pub tx_hex: String,
}

/// Represents the response for building a withdrawal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuildTransferalTransactionResponse {
    pub tx_hex: String,
}

/// Represents the response for submitting a withdrawal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitWithdrawalTransactionResponse {
    pub tx_hash: String,
}

/// Represents the response for submitting a withdrawal transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubmitTransferalTransactionResponse {
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
