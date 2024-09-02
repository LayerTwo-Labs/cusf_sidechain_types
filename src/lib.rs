use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub const MAIN_ADDRESS_LENGTH: usize = 20;
pub const ADDRESS_LENGTH: usize = 20;
pub const HASH_LENGTH: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutPoint {
    Regular {
        transaction_number: u64,
        output_number: u8,
    },
    Deposit {
        sequence_number: u64,
    },
}

impl Display for OutPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Regular {
                transaction_number,
                output_number,
            } => write!(f, "r:{}:{}", transaction_number, output_number),
            Self::Deposit { sequence_number } => write!(f, "d:{}", sequence_number),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Output {
    Regular {
        address: [u8; ADDRESS_LENGTH],
        value: u64,
    },
    Withdrawal {
        address: [u8; ADDRESS_LENGTH],
        // Must be P2PKH.
        main_address: [u8; MAIN_ADDRESS_LENGTH],
        value: u64,
        fee: u64,
    },
}

impl Output {
    pub fn total_value(&self) -> u64 {
        match self {
            Self::Regular { value, .. } => *value,
            Self::Withdrawal { value, fee, .. } => *value + *fee,
        }
    }

    pub fn address(&self) -> [u8; ADDRESS_LENGTH] {
        match self {
            Self::Regular { address, .. } => *address,
            Self::Withdrawal { address, .. } => *address,
        }
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Regular { address, value } => {
                let address = bs58::encode(&address).with_check().into_string();
                let value = bitcoin::Amount::from_sat(*value);
                write!(f, "{address}: {value}")
            }
            Self::Withdrawal {
                address,
                main_address,
                value,
                fee,
            } => {
                todo!();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<OutPoint>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    pub fn value_out(&self) -> u64 {
        self.outputs.iter().map(|output| output.total_value()).sum()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Header {
    pub prev_main_block_hash: [u8; HASH_LENGTH],
    pub prev_side_block_hash: [u8; HASH_LENGTH],
    pub merkle_root: [u8; HASH_LENGTH],
}

// 6000 withdrawals
//
// Deposits
// BMM
// Transactions
// Wihdrawals

impl Header {
    fn validate_transactions(&self, transactions: &[Transaction]) -> bool {
        // TODO: Make this into proper merkle root, not just hash of concatenated hashes.
        let merkle_root: [u8; HASH_LENGTH] = blake3::hash(
            &transactions
                .iter()
                .map(|transaction| transaction.hash())
                .collect::<Vec<_>>()
                .concat(),
        )
        .into();
        self.merkle_root == merkle_root
    }
}

pub trait Hashable
where
    Self: Serialize,
{
    fn hash(&self) -> [u8; HASH_LENGTH] {
        let bytes = bincode::serialize(self).unwrap();
        blake3::hash(&bytes).into()
    }
}

impl<T: Serialize> Hashable for T {}

/*
// Most significant bit is 1 when it is a deposit and 0 when not.
// If it is a deposit, the remaining 63 bits represent the sequence number.
// If it is a regular outpoint, the following 55 bits represent the transaction number, and the
// remaining 8 bits represent the index.
#[derive(Serialize, Deserialize)]
pub struct OutPointEncoded(u64);

impl Display for OutPointEncoded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_deposit() {
            write!(f, "d:{}", self.number())
        } else {
            write!(f, "r:{}:{}", self.number(), self.index())
        }
    }
}

impl OutPointEncoded {
    const DEPOSIT_MASK: u64 = 1u64.reverse_bits();
    const DEPOSIT_NUMBER_MASK: u64 = !Self::DEPOSIT_MASK;
    const INDEX_MASK: u64 = 0xFF;
    const TRANSACTION_NUMBER_MASK: u64 = !Self::DEPOSIT_MASK & !Self::INDEX_MASK;

    pub fn new(deposit: bool, number: u64, index: u8) -> Self {
        let mut payload = 0;
        if deposit {
            if number & Self::DEPOSIT_MASK != 0 {
                todo!();
            }
            payload = number | Self::DEPOSIT_MASK;
        } else {
            // If the 8 most significant bits are not 8, the transaction number is invalid.
            if number & Self::INDEX_MASK.reverse_bits() != 0 {
                todo!();
            }
            payload = number << 8;
            if payload & !Self::TRANSACTION_NUMBER_MASK != 0 {
                todo!();
            }
            payload |= index as u64;
        }
        Self(payload)
    }

    pub fn is_deposit(&self) -> bool {
        self.0 & Self::DEPOSIT_MASK != 0
    }

    // Deposit number if it is a deposit, transaction number if it is a regular output.
    pub fn number(&self) -> u64 {
        if self.is_deposit() {
            self.0 & Self::DEPOSIT_NUMBER_MASK
        } else {
            (self.0 & Self::TRANSACTION_NUMBER_MASK) >> 8
        }
    }

    pub fn index(&self) -> u8 {
        if self.is_deposit() {
            0
        } else {
            (self.0 & Self::INDEX_MASK) as u8
        }
    }
}
*/
