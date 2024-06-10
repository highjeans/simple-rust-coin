use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use sha2::{Digest, Sha512};
use crate::database::{get_frontier_block, get_mining_difficulty};
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub(crate) number: u32,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) hash: String,

    pub(crate) prev_hash: String,
    pub(crate) transactions: Vec<Transaction>,
    pub(crate) nonce: u64,
    pub(crate) timestamp: u64,
    pub(crate) miner: String
}

fn ceil_div(a: u32, b: u32) -> u32 {
    (a as f32 / b as f32).ceil() as u32
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, miner: String) -> Block {
        let prev_block_number_hash = match get_frontier_block() {
            Ok(block) => { (block.number as i64, block.hash)}
            Err(_) => {(-1, String::from("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"))}
        };
        Block {
            number: prev_block_number_hash.0 as u32 + 1,
            hash: String::new(),
            prev_hash: prev_block_number_hash.1,
            transactions: transactions.clone(),
            nonce: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            miner: miner.clone()
        }
    }

    pub fn mine(&mut self) -> Result<()> {
        loop {
            let serialised_block = serde_json::to_string(self)?;
            self.hash = hex::encode(&Sha512::digest(serialised_block)[..]);
            if self.verify_hash() {
                return Ok(());
            }
            self.hash = String::new();
            self.nonce += 1;
        }
    }

    pub fn verify_hash(&self) -> bool {
        let mut mining_difficulty = get_mining_difficulty();
        let num_chars = ceil_div(mining_difficulty, 4);
        let mut i = 0;
        for num in self.hash.chars() {
            if i > num_chars { // went past the number of characters needed to be checked, at this point they are all fine
                return true;
            }
            let hex_val = num as u8 - '0' as u8;
            if i+1 != num_chars {
                if hex_val != 0 {
                    return false;
                }
            }
            else {
                mining_difficulty %= 4;
                if mining_difficulty == 0 {
                    return true; // checked 4x bits, at this point they are all 0
                }
                let comp_val: u8 = 0b1000 >> (mining_difficulty - 1);
                return hex_val < comp_val;
            }
            i += 1;
        }
        false // if for whatever reason we get here
    }
}
