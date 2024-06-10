use std::time::{SystemTime, UNIX_EPOCH};
use crate::database::get_frontier_block;
use crate::transaction::Transaction;
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
}
