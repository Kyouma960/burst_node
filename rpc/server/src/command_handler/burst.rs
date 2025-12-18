// Burst-specific RPC command handlers

use crate::command_handler::RpcCommandHandler;
use rsnano_ledger::burst_helpers;
use rsnano_ledger::burst_verification;
use rsnano_rpc_messages::{
    BrnBalanceArgs, BrnBalanceResponse, RequestVerificationArgs, RequestVerificationResponse,
    TrstHistoryArgs, TrstHistoryEntry, TrstHistoryResponse, VoteVerificationArgs,
    VoteVerificationResponse,
};
use rsnano_types::{Account, Amount, UnixTimestamp, calculate_brn};

impl RpcCommandHandler {
    /// Get BRN balance for an account
    pub(crate) fn brn_balance(&self, args: BrnBalanceArgs) -> anyhow::Result<BrnBalanceResponse> {
        let store = &self.node.ledger.store;
        let txn = store.begin_read();

        // Get verification info
        let verification_info = burst_helpers::get_verification_info(store, &txn, &args.account);

        // Calculate BRN
        let verified_timestamp = verification_info
            .verified_timestamp
            .unwrap_or(verification_info.account_creation_timestamp);

        if verified_timestamp.as_u64() == 0 {
            return Ok(BrnBalanceResponse {
                brn: "0".to_string(),
            });
        }

        let current_timestamp = UnixTimestamp::now();
        let total_trst_created = burst_helpers::calculate_total_trst_created(store, &txn, &args.account);
        let current_brn = calculate_brn(verified_timestamp, current_timestamp, total_trst_created);

        Ok(BrnBalanceResponse {
            brn: current_brn.to_string(),
        })
    }

    /// Request verification for a wallet
    pub(crate) fn request_verification(
        &self,
        args: RequestVerificationArgs,
    ) -> anyhow::Result<RequestVerificationResponse> {
        let store = &self.node.ledger.store;
        let mut txn = store.begin_write();

        let account_creation_timestamp = args
            .account_creation_timestamp
            .map(UnixTimestamp::from)
            .unwrap_or_else(UnixTimestamp::now);

        burst_verification::request_verification(store, &mut txn, &args.account, account_creation_timestamp)?;

        txn.commit();

        // Get updated verification status
        let read_txn = store.begin_read();
        let verification_info = burst_helpers::get_verification_info(store, &read_txn, &args.account);

        let status_str = match verification_info.status {
            rsnano_types::VerificationStatus::Unverified => "unverified",
            rsnano_types::VerificationStatus::Pending => "pending",
            rsnano_types::VerificationStatus::Verified => "verified",
            rsnano_types::VerificationStatus::UnverifiedAfterRevote => "unverified",
        };

        Ok(RequestVerificationResponse {
            success: true,
            status: status_str.to_string(),
        })
    }

    /// Vote on verification (circle or random validator)
    pub(crate) fn vote_verification(
        &self,
        args: VoteVerificationArgs,
    ) -> anyhow::Result<VoteVerificationResponse> {
        let store = &self.node.ledger.store;
        let mut txn = store.begin_write();

        let result = match args.vote_type.as_str() {
            "circle" => {
                // Circle validator vote - burn BRN
                let brn_amount = args
                    .amount
                    .as_ref()
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or_else(|| anyhow::anyhow!("Invalid BRN amount"))?;

                let threshold_reached = burst_verification::circle_validator_vote(
                    store,
                    &mut txn,
                    &args.validator,
                    &args.candidate,
                    brn_amount,
                )?;

                VoteVerificationResponse {
                    success: true,
                    threshold_reached,
                    status: if threshold_reached {
                        "circle_threshold_reached".to_string()
                    } else {
                        "pending".to_string()
                    },
                }
            }
            "random" => {
                // Random validator vote - stake TRST
                let stake_amount = args
                    .amount
                    .as_ref()
                    .and_then(|s| s.parse::<u128>().ok())
                    .map(Amount::raw)
                    .ok_or_else(|| anyhow::anyhow!("Invalid TRST stake amount"))?;

                let vote: Option<bool> = args.vote.as_ref().and_then(|v| match v.as_str() {
                    "legitimate" => Some(true),
                    "illegitimate" => Some(false),
                    "neither" | _ => None,
                });

                let verification_result = burst_verification::random_validator_vote(
                    store,
                    &mut txn,
                    &args.validator,
                    &args.candidate,
                    vote,
                    stake_amount,
                )?;

                let (success, threshold_reached, status) = match verification_result {
                    burst_verification::VerificationResult::Pending => {
                        (true, false, "pending".to_string())
                    }
                    burst_verification::VerificationResult::Verified { .. } => {
                        (true, true, "verified".to_string())
                    }
                    burst_verification::VerificationResult::Unverified { .. } => {
                        (true, true, "unverified".to_string())
                    }
                };

                VoteVerificationResponse {
                    success,
                    threshold_reached,
                    status,
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid vote type. Must be 'circle' or 'random'"));
            }
        };

        txn.commit();
        Ok(result)
    }

    /// Get TRST transaction history for an account
    pub(crate) fn trst_history(
        &self,
        args: TrstHistoryArgs,
    ) -> anyhow::Result<TrstHistoryResponse> {
        let store = &self.node.ledger.store;
        let txn = store.begin_read();

        // Get account info to find the head block
        let account_info = match store.account.get(&txn, &args.account) {
            Some(info) => info,
            None => {
                return Ok(TrstHistoryResponse {
                    history: Vec::new(),
                });
            }
        };

        let mut history = Vec::new();
        let mut current_hash = account_info.head;
        let count = args.count.unwrap_or(100); // Default to 100 transactions
        let current_timestamp = UnixTimestamp::now();

        // Walk backwards through the chain
        while !current_hash.is_zero()
            && current_hash != account_info.open_block
            && history.len() < count as usize
        {
            if let Some(block) = store.block.get(&txn, &current_hash) {
                // Check if this block has Burst metadata
                if let Some(ref burst_meta) = block.sideband().burst_metadata {
                    let method_str = match burst_meta.method {
                        rsnano_types::TransactionMethod::Burn => "burn",
                        rsnano_types::TransactionMethod::Send => "send",
                        rsnano_types::TransactionMethod::Merge => "merge",
                    };

                    let is_transferable = burst_meta.is_transferable(current_timestamp);

                    history.push(TrstHistoryEntry {
                        hash: current_hash.to_string(),
                        method: method_str.to_string(),
                        amount: block.balance().number().to_string(),
                        epoch: burst_meta.epoch.map(|e| e.to_string()),
                        link: burst_meta.link.map(|l| l.to_string()),
                        timestamp: block.sideband().timestamp.as_u64() / 1000, // Convert milliseconds to seconds
                        expiry_timestamp: burst_meta.expiry_timestamp.map(|t| t.as_u64()),
                        is_orphaned: burst_meta.is_orphaned,
                        is_transferable,
                    });
                }

                current_hash = block.previous();
            } else {
                break;
            }
        }

        Ok(TrstHistoryResponse { history })
    }
}

