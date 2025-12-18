// Utilities for creating Burst transactions (burn, send, merge, split)

use crate::{
    Amount, BlockHash, UnixTimestamp, BurstBlockMetadata,
};

/// Create Burst metadata for a burn transaction
/// Burn transactions create TRST from BRN (1:1 ratio)
pub fn create_burn_metadata(
    block_hash: BlockHash,
    timestamp: UnixTimestamp,
) -> BurstBlockMetadata {
    BurstBlockMetadata::new_burn(timestamp, block_hash)
}

/// Create Burst metadata for a send transaction
/// Send transactions transfer TRST to another wallet
pub fn create_send_metadata(
    previous_link: BlockHash,
    previous_epoch: BlockHash,
    timestamp: UnixTimestamp,
) -> BurstBlockMetadata {
    BurstBlockMetadata::new_send(timestamp, previous_link, previous_epoch)
}

/// Create Burst metadata for a merge transaction
/// Merge transactions combine multiple TRST tokens into one
pub fn create_merge_metadata(
    link_list: Vec<BlockHash>,
    merge_hash: BlockHash,
    timestamp: UnixTimestamp,
    fastest_expiry: UnixTimestamp,
) -> BurstBlockMetadata {
    BurstBlockMetadata::new_merge(timestamp, link_list, fastest_expiry, merge_hash)
}

/// Calculate the total amount from a list of TRST transaction hashes
/// This is used when creating merge transactions
pub fn calculate_merge_amount(
    _link_list: &[BlockHash],
    // In a real implementation, this would query the ledger for each hash
    // and sum their amounts. For now, this is a placeholder.
    _get_block_amount: impl Fn(&BlockHash) -> Option<Amount>,
) -> Amount {
    // TODO: Implement actual ledger query
    // For now, return zero as placeholder
    Amount::ZERO
}

/// Split a TRST transaction into multiple send transactions
/// Each split maintains the same epoch and link as the original
pub fn create_split_metadata(
    original_link: BlockHash,
    original_epoch: BlockHash,
    timestamp: UnixTimestamp,
    _split_amounts: &[Amount], // Amounts for each split
) -> Vec<BurstBlockMetadata> {
    // For each split amount, create a send metadata with same epoch/link
    // In practice, this would be called once per split
    vec![BurstBlockMetadata::new_send(timestamp, original_link, original_epoch)]
}

/// Find the fastest expiring TRST from a list of transaction hashes
/// Used when merging TRST tokens with different expiry dates
pub fn find_fastest_expiry(
    _link_list: &[BlockHash],
    // In a real implementation, this would query the ledger for each hash
    // and find the minimum expiry timestamp
    _get_block_expiry: impl Fn(&BlockHash) -> Option<UnixTimestamp>,
) -> Option<UnixTimestamp> {
    // TODO: Implement actual ledger query
    // This will be implemented using burst_helpers::get_fastest_expiry
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_burn_metadata() {
        let hash = BlockHash::from(1);
        let timestamp = UnixTimestamp::from(1000);
        let metadata = create_burn_metadata(hash, timestamp);
        
        assert_eq!(metadata.method, TransactionMethod::Burn);
        assert_eq!(metadata.epoch, Some(hash));
        assert_eq!(metadata.link, None);
    }

    #[test]
    fn test_create_send_metadata() {
        let link = BlockHash::from(1);
        let epoch = BlockHash::from(2);
        let timestamp = UnixTimestamp::from(1000);
        let metadata = create_send_metadata(link, epoch, timestamp);
        
        assert_eq!(metadata.method, TransactionMethod::Send);
        assert_eq!(metadata.epoch, Some(epoch));
        assert_eq!(metadata.link, Some(link));
    }

    #[test]
    fn test_create_merge_metadata() {
        let link_list = vec![BlockHash::from(1), BlockHash::from(2)];
        let merge_hash = BlockHash::from(3);
        let timestamp = UnixTimestamp::from(1000);
        let fastest_expiry = UnixTimestamp::from(2000);
        let metadata = create_merge_metadata(link_list.clone(), merge_hash, timestamp, fastest_expiry);
        
        assert_eq!(metadata.method, TransactionMethod::Merge);
        assert_eq!(metadata.epoch, Some(merge_hash));
        assert_eq!(metadata.link_list, link_list);
        assert_eq!(metadata.expiry_timestamp, Some(fastest_expiry));
    }
}

