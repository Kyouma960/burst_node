// Burst-specific RPC messages

use serde::{Deserialize, Serialize};
use rsnano_types::Account;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BrnBalanceArgs {
    pub account: Account,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BrnBalanceResponse {
    pub brn: String, // BRN balance as string (u64)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RequestVerificationArgs {
    pub account: Account,
    pub account_creation_timestamp: Option<u64>, // Optional, defaults to current time
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RequestVerificationResponse {
    pub success: bool,
    pub status: String, // "pending", "verified", "unverified"
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct VoteVerificationArgs {
    pub validator: Account,
    pub candidate: Account,
    #[serde(rename = "type")]
    pub vote_type: String, // "circle" or "random"
    pub amount: Option<String>, // BRN amount for circle, TRST amount for random
    pub vote: Option<String>, // "legitimate", "illegitimate", or "neither" (for random only)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct VoteVerificationResponse {
    pub success: bool,
    pub threshold_reached: bool,
    pub status: String, // Current verification status
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TrstHistoryArgs {
    pub account: Account,
    pub count: Option<u32>, // Number of transactions to return
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TrstHistoryEntry {
    pub hash: String,
    pub method: String, // "burn", "send", "merge"
    pub amount: String,
    pub epoch: Option<String>,
    pub link: Option<String>,
    pub timestamp: u64,
    pub expiry_timestamp: Option<u64>,
    pub is_orphaned: bool,
    pub is_transferable: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TrstHistoryResponse {
    pub history: Vec<TrstHistoryEntry>,
}

