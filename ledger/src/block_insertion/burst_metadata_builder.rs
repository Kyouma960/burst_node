// Helper to derive Burst metadata from blocks and their chain

use burst_nullable_lmdb::Transaction;
use burst_store_lmdb::LmdbStore;
use burst_types::{
    Account, Block, BlockHash, BurstBlockMetadata, TransactionMethod, UnixTimestamp,
};
use crate::burst_helpers;

/// Derive Burst metadata for a block based on its type and chain history
/// This determines if a block is a burn, send, or merge transaction
pub fn derive_burst_metadata(
    store: &LmdbStore,
    txn: &dyn Transaction,
    block: &Block,
    account: &Account,
    previous_block: Option<&burst_types::SavedBlock>,
    current_timestamp: UnixTimestamp,
) -> Option<BurstBlockMetadata> {
    // Check if this account has any previous blocks
    // If no previous blocks exist, this is likely a burn transaction (first transaction)
    if previous_block.is_none() {
        // First transaction from this account - it's a burn
        return Some(BurstBlockMetadata::new_burn(
            current_timestamp,
            block.hash(),
        ));
    }

    // Check previous block to determine transaction type
    if let Some(prev) = previous_block {
        if let Some(ref prev_burst_meta) = prev.sideband().burst_metadata {
            // Previous block has Burst metadata - this is a send transaction
            // Link is the previous block's hash, epoch is the previous block's epoch
            if let Some(prev_epoch) = prev_burst_meta.epoch {
                return Some(BurstBlockMetadata::new_send(
                    current_timestamp,
                    prev.hash(),
                    prev_epoch,
                ));
            }
        } else {
            // Previous block doesn't have Burst metadata - check if it's a receive
            // If we received TRST, the next send should use the received block's epoch
            // For now, treat as a send with epoch from the source block
            if prev.sideband().details.is_receive {
                // This is a send after receiving - need to find the epoch from the source
                if let Some(source_hash) = prev.source() {
                    if let Some(source_block) = store.block.get(txn, &source_hash) {
                        if let Some(ref source_burst_meta) = source_block.sideband().burst_metadata {
                            if let Some(source_epoch) = source_burst_meta.epoch {
                                return Some(BurstBlockMetadata::new_send(
                                    current_timestamp,
                                    prev.hash(),
                                    source_epoch,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Default: treat as send transaction
    // In a full implementation, we'd need to detect merge transactions
    // which would require checking if multiple TRST are being combined
    None
}

/// Check if a block should be treated as a Burst transaction
pub fn is_burst_transaction(
    store: &LmdbStore,
    txn: &dyn Transaction,
    account: &Account,
) -> bool {
    // Check if account has verification info
    let verification_info = burst_helpers::get_verification_info(store, txn, account);
    
    // If account is verified or pending verification, it's a Burst account
    verification_info.status != burst_types::VerificationStatus::Unverified
        || !verification_info.circle_validator_votes.is_empty()
}

