// Post-insertion processing for Burst blocks
// Updates verification info and handles Burst-specific side effects

use burst_nullable_lmdb::WriteTransaction;
use burst_store_lmdb::LmdbStore;
use burst_types::{Account, BlockHash, TransactionMethod};
use crate::burst_helpers;

/// Process Burst-specific side effects after a block is inserted
pub fn process_burst_block_post_insert(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    account: &Account,
    block_hash: &BlockHash,
) -> anyhow::Result<()> {
    // Get the block we just inserted
    if let Some(block) = store.block.get(txn, block_hash) {
        if let Some(ref burst_meta) = block.sideband().burst_metadata {
            match burst_meta.method {
                TransactionMethod::Burn => {
                    // Update verification info: increment total TRST created
                    update_verification_info_after_burn(store, txn, account, block.balance().number())?;
                }
                TransactionMethod::Send => {
                    // Send transactions don't need special processing
                    // But we should check if the originator became unverified and orphan this TRST
                    if let Some(epoch) = burst_meta.epoch {
                        check_and_orphan_if_needed(store, txn, &epoch)?;
                    }
                }
                TransactionMethod::Merge => {
                    // Merge transactions: check if any merged TRST became orphaned
                    if let Some(epoch) = burst_meta.epoch {
                        check_and_orphan_if_needed(store, txn, &epoch)?;
                    }
                    // Also check all merged blocks
                    for link_hash in &burst_meta.link_list {
                        if let Some(merged_block) = store.block.get(txn, link_hash) {
                            if let Some(ref merged_meta) = merged_block.sideband().burst_metadata {
                                if let Some(merged_epoch) = merged_meta.epoch {
                                    check_and_orphan_if_needed(store, txn, &merged_epoch)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Update verification info after a burn transaction
fn update_verification_info_after_burn(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    account: &Account,
    _trst_amount: u128,
) -> anyhow::Result<()> {
    let verification_info = burst_helpers::get_verification_info(store, txn, account);
    
    // Note: total_trst_created is calculated on-the-fly, not stored
    // But we could store it for performance if needed
    // For now, we just ensure verification info exists
    
    store.verification.put(txn, account, &verification_info);
    Ok(())
}

/// Check if a TRST token's originator became unverified and orphan it if needed
fn check_and_orphan_if_needed(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    epoch: &BlockHash,
) -> anyhow::Result<()> {
    // Find the original burn block
    if let Some(burn_block) = store.block.get(txn, epoch) {
        let originator_account = burn_block.account();
        let verification_info = burst_helpers::get_verification_info(store, txn, &originator_account);
        
        // If originator is not verified, orphan all TRST from this epoch
        if !verification_info.status.can_transact() {
            // Find all blocks with this epoch and mark them as orphaned
            // This is expensive but necessary for correctness
            // In production, we might want to maintain an epoch index
            orphan_trst_by_epoch(store, txn, epoch)?;
        }
    }
    Ok(())
}

/// Orphan all TRST tokens that originated from a specific epoch
fn orphan_trst_by_epoch(
    _store: &LmdbStore,
    _txn: &mut WriteTransaction,
    _epoch: &BlockHash,
) -> anyhow::Result<()> {
    // This is a placeholder - full implementation would:
    // 1. Find all blocks with epoch == epoch_hash
    // 2. Mark them as orphaned
    // 3. For merge transactions, handle proportional orphaning
    
    // For now, we rely on the is_trst_orphaned check during validation
    // which checks the originator's verification status on-the-fly
    
    Ok(())
}

