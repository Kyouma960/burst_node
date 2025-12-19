// Burst-specific validation rules

use super::BlockValidator;
use crate::{BlockError, burst_helpers};
use burst_types::{
    TransactionMethod, UnixTimestamp,
};

impl<'a> BlockValidator<'a> {
    /// Validate Burst-specific rules for burn transactions
    pub(crate) fn ensure_valid_burst_burn(&self) -> Result<(), BlockError> {
        // Check if this is a Burst block with burn method
        let sideband = self.new_sideband();
        if let Some(ref burst_meta) = sideband.burst_metadata {
            if burst_meta.method == TransactionMethod::Burn {
                // Get store reference - if not available, skip Burst validation
                let store = match self.store {
                    Some(s) => s,
                    None => return Ok(()), // Not a Burst node or store not available
                };

                // Create a read transaction to query verification info
                let txn = store.begin_read();

                // Get verification info
                let verification_info = burst_helpers::get_verification_info(store, &txn, &self.account);

                // Check verification status
                if !verification_info.status.can_transact() {
                    return Err(BlockError::Unreceivable);
                }

                // Calculate BRN
                let verified_timestamp = verification_info
                    .verified_timestamp
                    .unwrap_or(verification_info.account_creation_timestamp);
                let current_timestamp = UnixTimestamp::from(self.now.as_u64() / 1000); // Convert milliseconds to seconds
                let total_trst_created = burst_helpers::calculate_total_trst_created(store, &txn, &self.account);
                let current_brn = burst_types::calculate_brn(verified_timestamp, current_timestamp, total_trst_created);

                // Check if BRN is sufficient
                let amount = self.block.balance_field().unwrap_or_default();
                let amount_u128 = amount.number();
                let current_brn_u128 = current_brn as u128;

                if current_brn_u128 < amount_u128 {
                    return Err(BlockError::BalanceMismatch);
                }
            }
        }
        Ok(())
    }

    /// Validate Burst-specific rules for send transactions
    pub(crate) fn ensure_valid_burst_send(&self) -> Result<(), BlockError> {
        let sideband = self.new_sideband();
        if let Some(ref burst_meta) = sideband.burst_metadata {
            if burst_meta.method == TransactionMethod::Send {
                let store = match self.store {
                    Some(s) => s,
                    None => return Ok(()),
                };

                let txn = store.begin_read();

                // Get verification info
                let verification_info = burst_helpers::get_verification_info(store, &txn, &self.account);

                // Check verification status
                if !verification_info.status.can_transact() {
                    return Err(BlockError::Unreceivable);
                }

                // Check if TRST is expired or orphaned
                let current_timestamp = UnixTimestamp::from(self.now.as_u64() / 1000);
                if !burst_meta.is_transferable(current_timestamp) {
                    return Err(BlockError::Unreceivable);
                }

                // Check if TRST is orphaned (originator is unverified)
                if let Some(epoch) = burst_meta.epoch {
                    if burst_helpers::is_trst_orphaned(store, &txn, &epoch) {
                        return Err(BlockError::Unreceivable);
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate Burst-specific rules for merge transactions
    pub(crate) fn ensure_valid_burst_merge(&self) -> Result<(), BlockError> {
        let sideband = self.new_sideband();
        if let Some(ref burst_meta) = sideband.burst_metadata {
            if burst_meta.method == TransactionMethod::Merge {
                let store = match self.store {
                    Some(s) => s,
                    None => return Ok(()),
                };

                let txn = store.begin_read();

                // Get verification info
                let verification_info = burst_helpers::get_verification_info(store, &txn, &self.account);

                // Check verification status
                if !verification_info.status.can_transact() {
                    return Err(BlockError::Unreceivable);
                }

                // Validate all merged transactions exist and are valid
                for link_hash in &burst_meta.link_list {
                    if !store.block.exists(&txn, link_hash) {
                        return Err(BlockError::GapSource);
                    }

                    // Check if merged TRST is transferable
                    if let Some(merged_block) = store.block.get(&txn, link_hash) {
                        if let Some(ref merged_meta) = merged_block.sideband().burst_metadata {
                            let current_timestamp = UnixTimestamp::from(self.now.as_u64() / 1000);
                            if !merged_meta.is_transferable(current_timestamp) {
                                return Err(BlockError::Unreceivable);
                            }

                            // Check if merged TRST is orphaned
                            if let Some(epoch) = merged_meta.epoch {
                                if burst_helpers::is_trst_orphaned(store, &txn, &epoch) {
                                    return Err(BlockError::Unreceivable);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

