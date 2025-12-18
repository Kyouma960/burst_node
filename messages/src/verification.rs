// Burst verification messages for network protocol

use super::MessageVariant;
use bitvec::prelude::BitArray;
use rsnano_types::{
    Account, Amount, BlockHash, DeserializationError, Signature, UnixTimestamp,
};
use std::io::{Read, Write};

/// Request verification for a wallet
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationRequest {
    /// Account requesting verification
    pub account: Account,
    /// Timestamp when account was created
    pub account_creation_timestamp: UnixTimestamp,
    /// Signature proving ownership of the account
    pub signature: Signature,
}

impl VerificationRequest {
    pub const SERIALIZED_SIZE: usize = Account::SERIALIZED_SIZE + 8 + Signature::SERIALIZED_SIZE;

    pub fn serialize<T>(&self, writer: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.account.serialize(writer)?;
        writer.write_all(&self.account_creation_timestamp.as_u64().to_be_bytes())?;
        self.signature.serialize(writer)?;
        Ok(())
    }

    pub fn deserialize<T>(reader: &mut T) -> Result<Self, DeserializationError>
    where
        T: Read,
    {
        let account = Account::deserialize(reader)?;
        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;
        let account_creation_timestamp = UnixTimestamp::from(u64::from_be_bytes(buffer));
        let signature = Signature::deserialize(reader)?;
        Ok(Self {
            account,
            account_creation_timestamp,
            signature,
        })
    }
}

/// Circle validator vote (burn BRN to vouch)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CircleValidatorVote {
    /// Validator account
    pub validator: Account,
    /// Candidate account being vouched for
    pub candidate: Account,
    /// Amount of BRN being burned
    pub brn_amount: u64,
    /// Signature from validator
    pub signature: Signature,
}

impl CircleValidatorVote {
    pub const SERIALIZED_SIZE: usize = Account::SERIALIZED_SIZE * 2 + 8 + Signature::SERIALIZED_SIZE;

    pub fn serialize<T>(&self, writer: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.validator.serialize(writer)?;
        self.candidate.serialize(writer)?;
        writer.write_all(&self.brn_amount.to_be_bytes())?;
        self.signature.serialize(writer)?;
        Ok(())
    }

    pub fn deserialize<T>(reader: &mut T) -> Result<Self, DeserializationError>
    where
        T: Read,
    {
        let validator = Account::deserialize(reader)?;
        let candidate = Account::deserialize(reader)?;
        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;
        let brn_amount = u64::from_be_bytes(buffer);
        let signature = Signature::deserialize(reader)?;
        Ok(Self {
            validator,
            candidate,
            brn_amount,
            signature,
        })
    }
}

/// Random validator vote (stake TRST and vote)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RandomValidatorVote {
    /// Validator account
    pub validator: Account,
    /// Candidate account being voted on
    pub candidate: Account,
    /// Vote: true = legitimate, false = illegitimate, None = neither
    /// Serialized as: 0 = None, 1 = Some(false), 2 = Some(true)
    pub vote: Option<bool>,
    /// Amount of TRST being staked
    pub stake_amount: Amount,
    /// Signature from validator
    pub signature: Signature,
}

impl RandomValidatorVote {
    pub const SERIALIZED_SIZE: usize = Account::SERIALIZED_SIZE * 2 + 1 + Amount::SERIALIZED_SIZE + Signature::SERIALIZED_SIZE;

    pub fn serialize<T>(&self, writer: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.validator.serialize(writer)?;
        self.candidate.serialize(writer)?;
        let vote_byte = match self.vote {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        };
        writer.write_all(&[vote_byte])?;
        self.stake_amount.serialize(writer)?;
        self.signature.serialize(writer)?;
        Ok(())
    }

    pub fn deserialize<T>(reader: &mut T) -> Result<Self, DeserializationError>
    where
        T: Read,
    {
        let validator = Account::deserialize(reader)?;
        let candidate = Account::deserialize(reader)?;
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        let vote = match buffer[0] {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            _ => return Err(DeserializationError::InvalidData),
        };
        let stake_amount = Amount::deserialize(reader)?;
        let signature = Signature::deserialize(reader)?;
        Ok(Self {
            validator,
            candidate,
            vote,
            stake_amount,
            signature,
        })
    }
}

/// Revote request (initiate revote for bad actor)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevoteRequest {
    /// Initiator account
    pub initiator: Account,
    /// Target account to revote on
    pub target: Account,
    /// Amount of BRN being staked for revote
    pub brn_stake: u64,
    /// Signature from initiator
    pub signature: Signature,
}

impl RevoteRequest {
    pub const SERIALIZED_SIZE: usize = Account::SERIALIZED_SIZE * 2 + 8 + Signature::SERIALIZED_SIZE;

    pub fn serialize<T>(&self, writer: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.initiator.serialize(writer)?;
        self.target.serialize(writer)?;
        writer.write_all(&self.brn_stake.to_be_bytes())?;
        self.signature.serialize(writer)?;
        Ok(())
    }

    pub fn deserialize<T>(reader: &mut T) -> Result<Self, DeserializationError>
    where
        T: Read,
    {
        let initiator = Account::deserialize(reader)?;
        let target = Account::deserialize(reader)?;
        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;
        let brn_stake = u64::from_be_bytes(buffer);
        let signature = Signature::deserialize(reader)?;
        Ok(Self {
            initiator,
            target,
            brn_stake,
            signature,
        })
    }
}

impl MessageVariant for VerificationRequest {
    fn header_extensions(&self, _payload_len: u16) -> BitArray<u16> {
        Default::default()
    }
}

impl MessageVariant for CircleValidatorVote {
    fn header_extensions(&self, _payload_len: u16) -> BitArray<u16> {
        Default::default()
    }
}

impl MessageVariant for RandomValidatorVote {
    fn header_extensions(&self, _payload_len: u16) -> BitArray<u16> {
        Default::default()
    }
}

impl MessageVariant for RevoteRequest {
    fn header_extensions(&self, _payload_len: u16) -> BitArray<u16> {
        Default::default()
    }
}

