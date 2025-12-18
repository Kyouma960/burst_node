// Helper functions for Burst-specific ledger operations

use burst_nullable_lmdb::Transaction;
use burst_store_lmdb::LmdbStore;
use burst_types::{
    Account, BlockHash, UnixTimestamp, VerificationInfo, calculate_brn,
};

/// Get verification info for an account, or return default (unverified)
pub fn get_verification_info(
    store: &LmdbStore,
    txn: &dyn Transaction,
    account: &Account,
) -> VerificationInfo {
    store
        .verification
        .get(txn, account)
        .unwrap_or_default()
}

/// Calculate total TRST created by an account (sum of all burn transaction amounts)
/// This walks through the account's block chain and sums burn transactions
pub fn calculate_total_trst_created(
    store: &LmdbStore,
    txn: &dyn Transaction,
    account: &Account,
) -> u64 {
    // Get account info to find the head block
    let account_info = match store.account.get(txn, account) {
        Some(info) => info,
        None => return 0, // Account doesn't exist
    };

    // Walk backwards through the chain from head to open block
    let mut current_hash = account_info.head;
    let mut total_trst = 0u128; // Use u128 to match Amount

    // Iterate through blocks until we reach the open block
    while !current_hash.is_zero() && current_hash != account_info.open_block {
        if let Some(block) = store.block.get(txn, &current_hash) {
            // Check if this is a burn transaction
            if let Some(ref burst_meta) = block.sideband().burst_metadata {
                if burst_meta.method == burst_types::TransactionMethod::Burn {
                    // Add the amount (balance difference indicates burn amount)
                    // For burn transactions, the amount is the balance of the burn block
                    total_trst += block.balance().number();
                }
            }

            // Move to previous block
            current_hash = block.previous();
        } else {
            break; // Block not found, stop iteration
        }
    }

    // Convert to u64 (may truncate, but BRN calculation uses u64)
    // In practice, this should be fine as BRN is limited by time
    total_trst.min(u64::MAX as u128) as u64
}

/// Check if a TRST token (identified by epoch) is orphaned
/// A TRST is orphaned if its originator account is unverified
pub fn is_trst_orphaned(
    store: &LmdbStore,
    txn: &dyn Transaction,
    epoch: &BlockHash,
) -> bool {
    // Find the original burn block
    if let Some(burn_block) = store.block.get(txn, epoch) {
        let originator_account = burn_block.account();
        let verification_info = get_verification_info(store, txn, &originator_account);
        
        // TRST is orphaned if originator is not verified
        !verification_info.status.can_transact()
    } else {
        // If we can't find the burn block, consider it orphaned
        true
    }
}

/// Get the fastest expiry timestamp from a list of TRST transactions
pub fn get_fastest_expiry(
    store: &LmdbStore,
    txn: &dyn Transaction,
    link_list: &[BlockHash],
) -> Option<UnixTimestamp> {
    let mut fastest_expiry: Option<UnixTimestamp> = None;

    for hash in link_list {
        if let Some(block) = store.block.get(txn, hash) {
            if let Some(ref burst_meta) = block.sideband().burst_metadata {
                if let Some(expiry) = burst_meta.expiry_timestamp {
                    match fastest_expiry {
                        None => fastest_expiry = Some(expiry),
                        Some(current) => {
                            if expiry < current {
                                fastest_expiry = Some(expiry);
                            }
                        }
                    }
                }
            }
        }
    }

    fastest_expiry
}

