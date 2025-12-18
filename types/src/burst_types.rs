// Burst-specific types for transaction methods, verification status, etc.

use crate::{BlockHash, UnixTimestamp};
use serde::{Deserialize, Serialize};

/// Transaction method type for Burst transactions
#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy, FromPrimitive, Serialize, Deserialize)]
pub enum TransactionMethod {
    /// Burn BRN to create TRST (1:1 ratio)
    Burn = 0,
    /// Send TRST to another wallet
    Send = 1,
    /// Merge multiple TRST tokens into one
    Merge = 2,
}

impl TransactionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionMethod::Burn => "burn",
            TransactionMethod::Send => "send",
            TransactionMethod::Merge => "merge",
        }
    }
}

// VerificationStatus and VerificationInfo are now in verification_info.rs

/// Burst-specific block metadata
/// This extends the base StateBlock with Burst fields
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurstBlockMetadata {
    /// Transaction method (burn, send, or merge)
    pub method: TransactionMethod,
    /// Epoch: hash of the original burn transaction that created this TRST
    /// For burn transactions, this is the block hash itself
    /// For send transactions, this is copied from the previous transaction
    /// For merge transactions, this becomes the merge transaction hash
    pub epoch: Option<BlockHash>,
    /// Link: hash of the previous TRST transaction this was derived from
    /// None for burn transactions
    /// For send transactions, this is the previous transaction's hash
    /// For merge transactions, this contains a list of all linked transactions
    pub link: Option<BlockHash>,
    /// Link list: for merge transactions, contains all transaction hashes being merged
    pub link_list: Vec<BlockHash>,
    /// Timestamp when this transaction was created
    pub timestamp: UnixTimestamp,
    /// Expiry timestamp (calculated as timestamp + TRST_EXPIRY_YEARS)
    /// Only relevant for TRST transactions
    pub expiry_timestamp: Option<UnixTimestamp>,
    /// Whether this TRST token is orphaned (untransferable)
    pub is_orphaned: bool,
}

impl BurstBlockMetadata {
    pub fn new_burn(timestamp: UnixTimestamp, block_hash: BlockHash) -> Self {
        let expiry = timestamp.as_u64() + crate::burst_constants::trst_expiry_seconds();
        Self {
            method: TransactionMethod::Burn,
            epoch: Some(block_hash),
            link: None,
            link_list: Vec::new(),
            timestamp,
            expiry_timestamp: Some(UnixTimestamp::from(expiry)),
            is_orphaned: false,
        }
    }

    pub fn new_send(
        timestamp: UnixTimestamp,
        previous_link: BlockHash,
        previous_epoch: BlockHash,
    ) -> Self {
        let expiry = timestamp.as_u64() + crate::burst_constants::trst_expiry_seconds();
        Self {
            method: TransactionMethod::Send,
            epoch: Some(previous_epoch),
            link: Some(previous_link),
            link_list: Vec::new(),
            timestamp,
            expiry_timestamp: Some(UnixTimestamp::from(expiry)),
            is_orphaned: false,
        }
    }

    pub fn new_merge(
        timestamp: UnixTimestamp,
        link_list: Vec<BlockHash>,
        fastest_expiry: UnixTimestamp,
        merge_hash: BlockHash,
    ) -> Self {
        Self {
            method: TransactionMethod::Merge,
            epoch: Some(merge_hash),
            link: None,
            link_list,
            timestamp,
            expiry_timestamp: Some(fastest_expiry),
            is_orphaned: false,
        }
    }

    /// Check if this TRST token has expired
    pub fn is_expired(&self, current_time: UnixTimestamp) -> bool {
        if let Some(expiry) = self.expiry_timestamp {
            current_time >= expiry
        } else {
            false
        }
    }

    /// Check if this transaction can be transferred
    pub fn is_transferable(&self, current_time: UnixTimestamp) -> bool {
        !self.is_orphaned && !self.is_expired(current_time)
    }
}

impl Default for BurstBlockMetadata {
    fn default() -> Self {
        Self {
            method: TransactionMethod::Send,
            epoch: None,
            link: None,
            link_list: Vec::new(),
            timestamp: UnixTimestamp::from(0),
            expiry_timestamp: None,
            is_orphaned: false,
        }
    }
}

// BRN calculation is in burst_validation.rs

