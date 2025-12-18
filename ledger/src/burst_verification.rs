// Burst verification voting system implementation
// Handles circle validators (burn BRN) and random validators (stake TRST)

use burst_nullable_lmdb::{Transaction, WriteTransaction};
use burst_store_lmdb::LmdbStore;
use burst_types::{
    Account, Amount, BlockHash, UnixTimestamp, VerificationInfo, VerificationStatus,
    CIRCLE_VALIDATOR_THRESHOLD, MIN_CIRCLE_VALIDATOR_STAKE_BRN, MIN_RANDOM_VALIDATOR_STAKE_TRST,
    RANDOM_VALIDATOR_THRESHOLD, calculate_brn,
};
use crate::burst_helpers;

/// Request verification for a wallet
/// This initiates the verification process
pub fn request_verification(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    account: &Account,
    account_creation_timestamp: UnixTimestamp,
) -> anyhow::Result<()> {
    // Create or update verification info
    let mut verification_info = store
        .verification
        .get(txn, account)
        .unwrap_or_else(|| VerificationInfo {
            account_creation_timestamp,
            ..VerificationInfo::default()
        });

    // Set status to pending if not already verified
    if verification_info.status == VerificationStatus::Unverified {
        verification_info.status = VerificationStatus::Pending;
        store.verification.put(txn, account, &verification_info);
    }

    Ok(())
}

/// Circle validator votes by burning BRN
/// Circle validators burn BRN to vouch for a wallet's legitimacy
pub fn circle_validator_vote(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    validator_account: &Account,
    candidate_account: &Account,
    brn_to_burn: u64,
) -> anyhow::Result<bool> {
    // Check if validator has enough BRN
    let read_txn = store.begin_read();
    let verification_info = burst_helpers::get_verification_info(store, &read_txn, validator_account);
    
    if !verification_info.status.can_transact() {
        return Err(anyhow::anyhow!("Validator account is not verified"));
    }

    // Calculate current BRN
    let verified_timestamp = verification_info
        .verified_timestamp
        .unwrap_or(verification_info.account_creation_timestamp);
    let current_timestamp = UnixTimestamp::now();
    let total_trst_created = burst_helpers::calculate_total_trst_created(store, &read_txn, validator_account);
    let current_brn = calculate_brn(verified_timestamp, current_timestamp, total_trst_created);

    // Check if validator has enough BRN
    if current_brn < brn_to_burn {
        return Err(anyhow::anyhow!("Insufficient BRN to burn"));
    }

    // Check minimum stake requirement
    if brn_to_burn < MIN_CIRCLE_VALIDATOR_STAKE_BRN {
        return Err(anyhow::anyhow!(
            "Must burn at least {} BRN",
            MIN_CIRCLE_VALIDATOR_STAKE_BRN
        ));
    }

    // Get candidate verification info
    let mut candidate_info = store
        .verification
        .get(txn, candidate_account)
        .unwrap_or_else(|| VerificationInfo {
            account_creation_timestamp: UnixTimestamp::now(),
            ..VerificationInfo::default()
        });

    // Add circle validator vote
    candidate_info.circle_validator_votes.insert(*validator_account);

    // Check if threshold is reached
    let threshold_reached = candidate_info.circle_validator_votes.len() >= CIRCLE_VALIDATOR_THRESHOLD as usize;

    // If threshold reached, move to random validator phase
    if threshold_reached && candidate_info.status == VerificationStatus::Pending {
        // Status remains Pending until random validators vote
        // The actual verification happens after random validator threshold is met
    }

    store.verification.put(txn, candidate_account, &candidate_info);

    Ok(threshold_reached)
}

/// Random validator votes by staking TRST
/// Random validators stake TRST and vote legitimate/illegitimate/neither
pub fn random_validator_vote(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    validator_account: &Account,
    candidate_account: &Account,
    vote: Option<bool>, // true = legitimate, false = illegitimate, None = neither
    stake_amount: Amount,
) -> anyhow::Result<VerificationResult> {
    // Check if validator is verified
    let read_txn = store.begin_read();
    let validator_info = burst_helpers::get_verification_info(store, &read_txn, validator_account);
    
    if !validator_info.status.can_transact() {
        return Err(anyhow::anyhow!("Validator account is not verified"));
    }

    // Check minimum stake requirement
    let stake_u128 = stake_amount.number();
    let min_stake_u128 = MIN_RANDOM_VALIDATOR_STAKE_TRST as u128;
    if stake_u128 < min_stake_u128 {
        return Err(anyhow::anyhow!(
            "Must stake at least {} TRST",
            MIN_RANDOM_VALIDATOR_STAKE_TRST
        ));
    }

    // Check if circle validator threshold is reached
    let mut candidate_info = store
        .verification
        .get(txn, candidate_account)
        .unwrap_or_else(|| VerificationInfo {
            account_creation_timestamp: UnixTimestamp::now(),
            ..VerificationInfo::default()
        });

    if candidate_info.circle_validator_votes.len() < CIRCLE_VALIDATOR_THRESHOLD as usize {
        return Err(anyhow::anyhow!("Circle validator threshold not yet reached"));
    }

    // Add random validator vote
    let stake_u64 = stake_amount.number().min(u64::MAX as u128) as u64;
    candidate_info.random_validator_votes.push(
        burst_types::RandomValidatorVote {
            validator: *validator_account,
            vote,
            stake_amount: stake_u64,
        }
    );

    // Check if we have enough random validators
    let random_vote_count = candidate_info.random_validator_votes.len();
    let threshold_reached = random_vote_count >= RANDOM_VALIDATOR_THRESHOLD as usize;

    let result = if threshold_reached {
        // Count votes
        let legitimate_votes: u64 = candidate_info
            .random_validator_votes
            .iter()
            .filter(|v| v.vote == Some(true))
            .map(|v| v.stake_amount)
            .sum();

        let illegitimate_votes: u64 = candidate_info
            .random_validator_votes
            .iter()
            .filter(|v| v.vote == Some(false))
            .map(|v| v.stake_amount)
            .sum();

        // Determine verification result
        if legitimate_votes > illegitimate_votes {
            // Wallet is verified
            candidate_info.status = VerificationStatus::Verified;
            candidate_info.verified_timestamp = Some(UnixTimestamp::now());
            
            // Distribute stakes: winners get their stake back plus share of losers' stakes
            VerificationResult::Verified {
                legitimate_stake: legitimate_votes,
                illegitimate_stake: illegitimate_votes,
            }
        } else if illegitimate_votes > legitimate_votes {
            // Wallet remains unverified
            candidate_info.status = VerificationStatus::Unverified;
            
            VerificationResult::Unverified {
                legitimate_stake: legitimate_votes,
                illegitimate_stake: illegitimate_votes,
            }
        } else {
            // Tie - remains pending (shouldn't happen with odd number of validators)
            VerificationResult::Pending
        }
    } else {
        VerificationResult::Pending
    };

    store.verification.put(txn, candidate_account, &candidate_info);

    Ok(result)
}

/// Result of verification voting
#[derive(Debug, Clone)]
pub enum VerificationResult {
    /// Verification is still pending (not enough votes yet)
    Pending,
    /// Wallet was verified (legitimate votes won)
    Verified {
        legitimate_stake: u64,
        illegitimate_stake: u64,
    },
    /// Wallet was not verified (illegitimate votes won)
    Unverified {
        legitimate_stake: u64,
        illegitimate_stake: u64,
    },
}

/// Initiate a revote for a potentially bad actor
/// Any verified wallet can initiate a revote by staking BRN
pub fn initiate_revote(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    initiator_account: &Account,
    target_account: &Account,
    brn_to_stake: u64,
) -> anyhow::Result<()> {
    // Check if initiator is verified
    let read_txn = store.begin_read();
    let initiator_info = burst_helpers::get_verification_info(store, &read_txn, initiator_account);
    
    if !initiator_info.status.can_transact() {
        return Err(anyhow::anyhow!("Initiator account is not verified"));
    }

    // Check if initiator has enough BRN
    let verified_timestamp = initiator_info
        .verified_timestamp
        .unwrap_or(initiator_info.account_creation_timestamp);
    let current_timestamp = UnixTimestamp::now();
    let total_trst_created = burst_helpers::calculate_total_trst_created(store, &read_txn, initiator_account);
    let current_brn = calculate_brn(verified_timestamp, current_timestamp, total_trst_created);

    if current_brn < brn_to_stake {
        return Err(anyhow::anyhow!("Insufficient BRN to stake"));
    }

    // Get target verification info
    let mut target_info = store
        .verification
        .get(txn, target_account)
        .unwrap_or_else(|| VerificationInfo {
            account_creation_timestamp: UnixTimestamp::now(),
            ..VerificationInfo::default()
        });

    // Reset verification status to pending for revote
    if target_info.status == VerificationStatus::Verified {
        target_info.status = VerificationStatus::Pending;
        target_info.verified_timestamp = None;
        // Clear previous votes to start fresh
        target_info.circle_validator_votes.clear();
        target_info.random_validator_votes.clear();
        
        store.verification.put(txn, target_account, &target_info);
    }

    Ok(())
}

/// Process revote result - if wallet becomes unverified, orphan all its TRST
pub fn process_revote_result(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    account: &Account,
    verified: bool,
) -> anyhow::Result<()> {
    let mut verification_info = store
        .verification
        .get(txn, account)
        .unwrap_or_default();

    if verified {
        verification_info.status = VerificationStatus::Verified;
        verification_info.verified_timestamp = Some(UnixTimestamp::now());
        
        // Unorphan any previously orphaned TRST
        crate::burst_orphaning::unorphan_wallet_trst(store, txn, account)?;
    } else {
        verification_info.status = VerificationStatus::UnverifiedAfterRevote;
        verification_info.verified_timestamp = None;
        
        // Orphan all TRST from this wallet
        crate::burst_orphaning::orphan_wallet_trst(store, txn, account)?;
    }

    store.verification.put(txn, account, &verification_info);

    Ok(())
}

/// Opt-in to become a random validator
/// Random validators can vote on verification requests
pub fn opt_in_random_validator(
    store: &LmdbStore,
    txn: &mut WriteTransaction,
    account: &Account,
) -> anyhow::Result<()> {
    // Check if account is verified
    let read_txn = store.begin_read();
    let verification_info = burst_helpers::get_verification_info(store, &read_txn, account);
    
    if !verification_info.status.can_transact() {
        return Err(anyhow::anyhow!("Account must be verified to become a random validator"));
    }

    // In a real implementation, we'd store a list of random validators
    // For now, any verified account can vote as a random validator
    // This could be stored in a separate database table
    
    Ok(())
}

