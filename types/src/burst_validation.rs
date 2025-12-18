// Burst transaction validation and BRN calculation logic

use crate::{
    Account, Amount, BlockHash, UnixTimestamp,
    verification_info::{VerificationInfo, VerificationStatus},
    BRN_INCREMENT_RATE_PER_MINUTE, trst_expiry_seconds,
};

/// Calculate BRN for a wallet
/// Formula: BRN = (time_since_verification_in_minutes * rate) - total_TRST_ever_created
pub fn calculate_brn(
    verified_timestamp: UnixTimestamp,
    current_timestamp: UnixTimestamp,
    total_trst_created: u64,
) -> u64 {
    if verified_timestamp.as_u64() == 0 {
        return 0; // Not verified, no BRN
    }

    let time_elapsed_seconds = current_timestamp.as_u64().saturating_sub(verified_timestamp.as_u64());
    let time_elapsed_minutes = time_elapsed_seconds / 60;
    
    let brn_accrued = time_elapsed_minutes * BRN_INCREMENT_RATE_PER_MINUTE;
    
    brn_accrued.saturating_sub(total_trst_created)
}

/// Calculate total TRST created by a wallet from its transaction history
/// This should be called by nodes to validate burn transactions
pub fn calculate_total_trst_created(
    _account: &Account,
    // In a real implementation, this would query the ledger for all burn transactions
    // from this account and sum their amounts
    _ledger: &dyn LedgerQuery, // Placeholder - will be implemented with actual ledger
) -> u64 {
    // TODO: Implement actual ledger query
    // For now, return 0 as placeholder
    0
}

/// Trait for querying ledger information (to be implemented by ledger)
pub trait LedgerQuery {
    /// Get all burn transactions for an account
    fn get_burn_transactions(&self, account: &Account) -> Vec<BurnTransaction>;
    
    /// Get verification info for an account
    fn get_verification_info(&self, account: &Account) -> Option<VerificationInfo>;
    
    /// Get current timestamp
    fn current_timestamp(&self) -> UnixTimestamp;
}

/// Represents a burn transaction
pub struct BurnTransaction {
    pub hash: BlockHash,
    pub amount: Amount,
    pub timestamp: UnixTimestamp,
}

/// Validation result for a transaction
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

/// Validate a burn transaction
/// Checks:
/// 1. Wallet is verified
/// 2. BRN >= amount to burn
pub fn validate_burn_transaction(
    account: &Account,
    amount: Amount,
    ledger: &dyn LedgerQuery,
) -> ValidationResult {
    // Check verification status
    let verification_info = match ledger.get_verification_info(account) {
        Some(info) => info,
        None => {
            return ValidationResult::Invalid("Account not found".to_string());
        }
    };

    if !verification_info.status.can_transact() {
        return ValidationResult::Invalid(format!(
            "Account is not verified (status: {:?})",
            verification_info.status
        ));
    }

    // Calculate current BRN
    let verified_timestamp = verification_info
        .verified_timestamp
        .unwrap_or(verification_info.account_creation_timestamp);
    let current_timestamp = ledger.current_timestamp();
    let total_trst_created = calculate_total_trst_created(account, ledger);
    let current_brn = calculate_brn(verified_timestamp, current_timestamp, total_trst_created);

    // Check if BRN is sufficient
    let amount_u128 = amount.number();
    // Convert BRN to u128 for comparison (BRN is u64, Amount is u128)
    let current_brn_u128 = current_brn as u128;
    if current_brn_u128 < amount_u128 {
        return ValidationResult::Invalid(format!(
            "Insufficient BRN: have {}, need {}",
            current_brn, amount_u128
        ));
    }

    ValidationResult::Valid
}

/// Validate a TRST send transaction
/// Checks:
/// 1. Source wallet is verified
/// 2. TRST token is not expired
/// 3. TRST token is not orphaned
pub fn validate_send_transaction(
    source_account: &Account,
    _trst_epoch: BlockHash,
    trst_timestamp: UnixTimestamp,
    is_orphaned: bool,
    ledger: &dyn LedgerQuery,
) -> ValidationResult {
    // Check verification status
    let verification_info = match ledger.get_verification_info(source_account) {
        Some(info) => info,
        None => {
            return ValidationResult::Invalid("Account not found".to_string());
        }
    };

    if !verification_info.status.can_transact() {
        return ValidationResult::Invalid(format!(
            "Account is not verified (status: {:?})",
            verification_info.status
        ));
    }

    // Check if TRST is orphaned
    if is_orphaned {
        return ValidationResult::Invalid("TRST token is orphaned".to_string());
    }

    // Check if TRST is expired
    let expiry_timestamp = trst_timestamp.as_u64() + trst_expiry_seconds();
    let current_timestamp = ledger.current_timestamp();
    if current_timestamp.as_u64() >= expiry_timestamp {
        return ValidationResult::Invalid("TRST token has expired".to_string());
    }

    ValidationResult::Valid
}

/// Check if a TRST token has expired
pub fn is_trst_expired(trst_timestamp: UnixTimestamp, current_timestamp: UnixTimestamp) -> bool {
    let expiry_timestamp = trst_timestamp.as_u64() + trst_expiry_seconds();
    current_timestamp.as_u64() >= expiry_timestamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_brn() {
        let verified = UnixTimestamp::from(1000);
        let current = UnixTimestamp::from(1000 + 60 * 100); // 100 minutes later
        let trst_created = 50;
        
        let brn = calculate_brn(verified, current, trst_created);
        assert_eq!(brn, 50); // 100 minutes * 1 - 50 = 50
    }

    #[test]
    fn test_brn_no_verification() {
        let verified = UnixTimestamp::from(0);
        let current = UnixTimestamp::from(1000);
        let trst_created = 0;
        
        let brn = calculate_brn(verified, current, trst_created);
        assert_eq!(brn, 0);
    }

    #[test]
    fn test_is_trst_expired() {
        let trst_timestamp = UnixTimestamp::from(1000);
        let expiry_seconds = trst_expiry_seconds();
        
        // Not expired
        let current_not_expired = UnixTimestamp::from(1000 + expiry_seconds - 1);
        assert!(!is_trst_expired(trst_timestamp, current_not_expired));
        
        // Expired
        let current_expired = UnixTimestamp::from(1000 + expiry_seconds);
        assert!(is_trst_expired(trst_timestamp, current_expired));
    }
}

