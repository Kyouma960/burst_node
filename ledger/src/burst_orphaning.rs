// Logic for handling illegitimate tokens and orphaning TRST

use rsnano_nullable_lmdb::{Transaction, WriteTransaction};
use rsnano_store_lmdb::LmdbStore;
use rsnano_types::{Account, BlockHash};

/// Mark a wallet as illegitimate and orphan all its TRST tokens
/// This should be called when a wallet is unverified after a revote
pub fn orphan_wallet_trst(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    account: &Account,
) -> anyhow::Result<Vec<BlockHash>> {
    // Get account info to find all blocks
    let account_info = match store.account.get(txn, account) {
        Some(info) => info,
        None => return Ok(Vec::new()), // Account doesn't exist
    };

    let mut orphaned_hashes = Vec::new();
    let mut current_hash = account_info.head;

    // Walk through all blocks for this account
    while !current_hash.is_zero() && current_hash != account_info.open_block {
        if let Some(block) = store.block.get(txn, &current_hash) {
            // Check if block has Burst metadata and is not already orphaned
            if let Some(ref burst_meta) = block.sideband().burst_metadata {
                if !burst_meta.is_orphaned {
                    // Create updated sideband with orphaned flag
                    let mut updated_sideband = block.sideband().clone();
                    if let Some(ref mut meta) = updated_sideband.burst_metadata {
                        meta.is_orphaned = true;
                    }
                    
                    // Create new SavedBlock with updated sideband
                    let updated_block = block.with_sideband(updated_sideband);
                    orphaned_hashes.push(current_hash);
                    
                    // Update the block in the store
                    store.block.put(txn, &updated_block);
                }
            }

            current_hash = block.previous();
        } else {
            break;
        }
    }

    Ok(orphaned_hashes)
}

/// Handle orphaning for merged TRST tokens
/// When a merged token contains illegitimate TRST, we need to mark
/// all transactions that have this merge as their epoch as partially orphaned
pub fn orphan_merged_trst_proportional(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    merge_hash: BlockHash,
    illegitimate_ratio: f64, // Ratio of illegitimate TRST in the merge (0.0 to 1.0)
) -> anyhow::Result<Vec<BlockHash>> {
    // Find all transactions that have this merge as their epoch
    // This requires iterating through all blocks (expensive, but necessary)
    // In production, we might want to maintain an index of epoch -> transactions
    
    let mut orphaned_hashes = Vec::new();
    
    // For now, this is a placeholder
    // Full implementation would:
    // 1. Find all blocks with epoch == merge_hash
    // 2. Mark them as partially orphaned (we'd need to add a field for partial orphaning)
    // 3. Or mark them as fully orphaned if ratio is 1.0
    
    // TODO: Implement epoch index or full scan
    // This is expensive but necessary for correctness
    
    Ok(orphaned_hashes)
}

/// Check if a TRST token is orphaned by checking its originator
pub fn check_trst_orphaned(
    store: &LmdbStore,
    txn: &dyn Transaction,
    epoch: &BlockHash,
) -> bool {
    // Find the original burn block
    if let Some(burn_block) = store.block.get(txn, epoch) {
        let originator_account = burn_block.account();
        
        // Check verification status
        if let Some(verification_info) = store.verification.get(txn, &originator_account) {
            return !verification_info.status.can_transact();
        }
        
        // If no verification info, consider it unverified (orphaned)
        true
    } else {
        // Can't find burn block, consider orphaned
        true
    }
}

/// Unorphan TRST tokens when a wallet is re-verified
pub fn unorphan_wallet_trst(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    account: &Account,
) -> anyhow::Result<Vec<BlockHash>> {
    let account_info = match store.account.get(txn, account) {
        Some(info) => info,
        None => return Ok(Vec::new()),
    };

    let mut unorphaned_hashes = Vec::new();
    let mut current_hash = account_info.head;

    while !current_hash.is_zero() && current_hash != account_info.open_block {
        if let Some(block) = store.block.get(txn, &current_hash) {
            if let Some(ref burst_meta) = block.sideband().burst_metadata {
                if burst_meta.is_orphaned {
                    // Create updated sideband without orphaned flag
                    let mut updated_sideband = block.sideband().clone();
                    if let Some(ref mut meta) = updated_sideband.burst_metadata {
                        meta.is_orphaned = false;
                    }
                    
                    // Create new SavedBlock with updated sideband
                    let updated_block = block.with_sideband(updated_sideband);
                    unorphaned_hashes.push(current_hash);
                    
                    // Update the block in the store
                    store.block.put(txn, &updated_block);
                }
            }

            current_hash = block.previous();
        } else {
            break;
        }
    }

    Ok(unorphaned_hashes)
}

