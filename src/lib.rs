use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const BLOCK_SIZE_LIMIT: usize = 1024 * 1024; // 1 MB by default.

pub const MAIN_ADDRESS_LENGTH: usize = 20;
pub const ADDRESS_LENGTH: usize = 20;
pub const HASH_LENGTH: usize = 32;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum OutPoint {
    Regular {
        transaction_number: u64,
        output_number: u8,
    },
    Coinbase {
        block_number: u32,
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
            } => write!(f, "r:{transaction_number}:{output_number}"),
            Self::Coinbase {
                block_number,
                output_number,
            } => write!(f, "c:{block_number}:{output_number}"),
            Self::Deposit { sequence_number } => write!(f, "d:{sequence_number}"),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
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
    pub fn compute_merkle_root(
        coinbase: &[Output],
        transactions: &[Transaction],
    ) -> [u8; HASH_LENGTH] {
        // TODO: Make this into proper merkle root, not just hash of concatenated hashes.
        let merkle_root: [u8; HASH_LENGTH] = blake3::hash(
            &[
                vec![coinbase.hash()],
                transactions
                    .iter()
                    .map(|transaction| transaction.hash())
                    .collect::<Vec<_>>(),
            ]
            .concat()
            .concat(),
        )
        .into();
        merkle_root
    }

    fn validate_block(&self, coinbase: &[Output], transactions: &[Transaction]) -> bool {
        let merkle_root = Self::compute_merkle_root(coinbase, transactions);
        self.merkle_root == merkle_root
    }
}

pub struct MainBlock {
    pub block_height: u32,
    pub block_hash: [u8; HASH_LENGTH],
    pub deposits: Vec<(OutPoint, Output)>,
    pub withdrawal_bundle_event: Option<WithdrawalBundleEvent>,
    pub bmm_hashes: Vec<[u8; HASH_LENGTH]>,
}

pub struct WithdrawalBundleEvent {
    pub withdrawal_bundle_event_type: WithdrawalBundleEventType,
    pub bmm_hash: [u8; HASH_LENGTH],
}

pub enum WithdrawalBundleEventType {
    Submitted,
    Succeded,
    Failed,
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
