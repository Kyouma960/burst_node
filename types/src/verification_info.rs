use crate::{Account, DeserializationError, UnixTimestamp, read_u8, read_u64_ne};
use num::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Read;

/// Verification status of a wallet
#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy, FromPrimitive, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Wallet is not verified (cannot accrue BRN or transact)
    Unverified = 0,
    /// Verification is pending (circle/random validators voting)
    Pending = 1,
    /// Wallet is verified (can accrue BRN and transact)
    Verified = 2,
    /// Wallet was verified but then unverified due to bad actor detection
    UnverifiedAfterRevote = 3,
}

impl VerificationStatus {
    pub fn can_accrue_brn(&self) -> bool {
        matches!(self, VerificationStatus::Verified)
    }

    pub fn can_transact(&self) -> bool {
        matches!(self, VerificationStatus::Verified)
    }
}

/// Vote from a random validator
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomValidatorVote {
    /// Validator account address
    pub validator: Account,
    /// Vote: true = legitimate, false = illegitimate, None = neither
    pub vote: Option<bool>,
    /// Amount staked (in TRST)
    pub stake_amount: u64,
}

/// Information about a wallet's verification status
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationInfo {
    /// Current verification status
    pub status: VerificationStatus,
    /// Timestamp when wallet was verified (for BRN calculation)
    /// None if wallet is not verified
    pub verified_timestamp: Option<UnixTimestamp>,
    /// Timestamp when account was created
    pub account_creation_timestamp: UnixTimestamp,
    /// Circle validator votes (set of validator account addresses who burned BRN)
    pub circle_validator_votes: HashSet<Account>,
    /// Random validator votes
    pub random_validator_votes: Vec<RandomValidatorVote>,
}

impl Default for VerificationInfo {
    fn default() -> Self {
        Self {
            status: VerificationStatus::Unverified,
            verified_timestamp: None,
            account_creation_timestamp: UnixTimestamp::from(0),
            circle_validator_votes: HashSet::new(),
            random_validator_votes: Vec::new(),
        }
    }
}

impl VerificationInfo {
    pub fn serialized_size(&self) -> usize {
        let mut size = 1; // status
        size += 1; // verified_timestamp present flag
        if self.verified_timestamp.is_some() {
            size += 8; // verified_timestamp
        }
        size += 8; // account_creation_timestamp
        size += 4; // circle_validator_votes count
        size += self.circle_validator_votes.len() * Account::SERIALIZED_SIZE;
        size += 4; // random_validator_votes count
        for _vote in &self.random_validator_votes {
            size += Account::SERIALIZED_SIZE; // validator
            size += 1; // vote (1 byte: 0=None, 1=Some(false), 2=Some(true))
            size += 8; // stake_amount
        }
        size
    }

    pub fn serialize<T>(&self, writer: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        writer.write_all(&[self.status as u8])?;
        
        // verified_timestamp
        if let Some(ts) = self.verified_timestamp {
            writer.write_all(&[1])?; // present flag
            writer.write_all(&ts.as_u64().to_ne_bytes())?;
        } else {
            writer.write_all(&[0])?; // absent flag
        }
        
        // account_creation_timestamp
        writer.write_all(&self.account_creation_timestamp.as_u64().to_ne_bytes())?;
        
        // circle_validator_votes
        writer.write_all(&(self.circle_validator_votes.len() as u32).to_ne_bytes())?;
        for account in &self.circle_validator_votes {
            account.serialize(writer)?;
        }
        
        // random_validator_votes
        writer.write_all(&(self.random_validator_votes.len() as u32).to_ne_bytes())?;
        for vote in &self.random_validator_votes {
            vote.validator.serialize(writer)?;
            let vote_byte = match vote.vote {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            };
            writer.write_all(&[vote_byte])?;
            writer.write_all(&vote.stake_amount.to_ne_bytes())?;
        }
        
        Ok(())
    }

    pub fn deserialize<T>(reader: &mut T) -> Result<Self, DeserializationError>
    where
        T: Read,
    {
        let status_byte = read_u8(reader)?;
        let status = VerificationStatus::from_u8(status_byte)
            .ok_or(DeserializationError::InvalidData)?;
        
        // verified_timestamp
        let verified_timestamp = if read_u8(reader)? == 1 {
            Some(UnixTimestamp::from(read_u64_ne(reader)?))
        } else {
            None
        };
        
        // account_creation_timestamp
        let account_creation_timestamp = UnixTimestamp::from(read_u64_ne(reader)?);
        
        // circle_validator_votes
        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        let circle_count = u32::from_ne_bytes(buffer);
        let mut circle_validator_votes = HashSet::new();
        for _ in 0..circle_count {
            circle_validator_votes.insert(Account::deserialize(reader)?);
        }
        
        // random_validator_votes
        reader.read_exact(&mut buffer)?;
        let random_count = u32::from_ne_bytes(buffer);
        let mut random_validator_votes = Vec::new();
        for _ in 0..random_count {
            let validator = Account::deserialize(reader)?;
            let vote_byte = read_u8(reader)?;
            let vote = match vote_byte {
                0 => None,
                1 => Some(false),
                2 => Some(true),
                _ => return Err(DeserializationError::InvalidData),
            };
            let stake_amount = read_u64_ne(reader)?;
            random_validator_votes.push(RandomValidatorVote {
                validator,
                vote,
                stake_amount,
            });
        }
        
        Ok(Self {
            status,
            verified_timestamp,
            account_creation_timestamp,
            circle_validator_votes,
            random_validator_votes,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.serialize(&mut buffer).expect("Should serialize verification info");
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let mut info = VerificationInfo::default();
        info.account_creation_timestamp = UnixTimestamp::from(1000);
        info.circle_validator_votes.insert(Account::from(1));
        info.random_validator_votes.push(RandomValidatorVote {
            validator: Account::from(2),
            vote: Some(true),
            stake_amount: 100,
        });
        
        let bytes = info.to_bytes();
        let deserialized = VerificationInfo::deserialize(&mut bytes.as_slice()).unwrap();
        assert_eq!(info, deserialized);
    }
}

