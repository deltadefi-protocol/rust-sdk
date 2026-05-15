use serde::Serialize;
use whisky::Asset;

use crate::{Interval, OrderRecordStatus, Symbol, TransferalType};

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

/// Request for getting aggregated price data.
#[derive(Debug, Serialize)]
pub struct GetAggregatedPriceRequest {
    pub symbol: Symbol,
    pub interval: Interval,
    pub start: i64,
    pub end: i64,
}

/// Request body for `POST /accounts/withdrawal/build`.
///
/// Use [`BuildWithdrawalTransactionRequest::new`] for a standard user withdrawal,
/// or [`BuildWithdrawalTransactionRequest::new_vault`] for an owner-only vault withdrawal.
#[derive(Debug, Serialize, Default, Clone)]
pub struct BuildWithdrawalTransactionRequest {
    pub withdrawal_amount: Vec<Asset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdraw_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account_id: Option<String>,
}

impl BuildWithdrawalTransactionRequest {
    /// Standard user withdrawal.
    pub fn new(withdrawal_amount: Vec<Asset>) -> Self {
        Self {
            withdrawal_amount,
            withdraw_type: None,
            vault_id: None,
            from_account_id: None,
        }
    }

    /// Vault withdrawal (owner-only). Sets `withdraw_type = "vault"`.
    pub fn new_vault(withdrawal_amount: Vec<Asset>, vault_id: impl Into<String>) -> Self {
        Self {
            withdrawal_amount,
            withdraw_type: Some("vault".to_string()),
            vault_id: Some(vault_id.into()),
            from_account_id: None,
        }
    }

    /// Set the vault sub-account to spend from (vault withdrawal only).
    pub fn with_from_account_id(mut self, from_account_id: impl Into<String>) -> Self {
        self.from_account_id = Some(from_account_id.into());
        self
    }
}

/// Request body for `POST /accounts/transferal/build`.
///
/// Either `to_address` or `to_account_id` must be set. Use
/// [`BuildTransferalTransactionRequest::to_address`] or
/// [`BuildTransferalTransactionRequest::to_account_id`] to construct.
#[derive(Debug, Serialize, Default, Clone)]
pub struct BuildTransferalTransactionRequest {
    pub transferal_amount: Vec<Asset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transferal_type: Option<TransferalType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account_id: Option<String>,
}

impl BuildTransferalTransactionRequest {
    /// Transfer to a wallet address.
    pub fn to_address(transferal_amount: Vec<Asset>, to_address: impl Into<String>) -> Self {
        Self {
            transferal_amount,
            to_address: Some(to_address.into()),
            ..Default::default()
        }
    }

    /// Transfer to a DeltaDeFi account by ID.
    pub fn to_account_id(transferal_amount: Vec<Asset>, to_account_id: impl Into<String>) -> Self {
        Self {
            transferal_amount,
            to_account_id: Some(to_account_id.into()),
            ..Default::default()
        }
    }

    /// Override the transferal type (defaults to `normal` server-side).
    /// `TransferalType::Rebate` is not accepted by this endpoint.
    pub fn with_transferal_type(mut self, transferal_type: TransferalType) -> Self {
        self.transferal_type = Some(transferal_type);
        self
    }

    /// Transfer out of a vault account (owner-only).
    pub fn with_vault_id(mut self, vault_id: impl Into<String>) -> Self {
        self.vault_id = Some(vault_id.into());
        self
    }

    /// Set the vault sub-account to spend from (vault transferal only).
    pub fn with_from_account_id(mut self, from_account_id: impl Into<String>) -> Self {
        self.from_account_id = Some(from_account_id.into());
        self
    }
}

