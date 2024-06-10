use sqlite;
use sqlite::Statement;
use crate::block::Block;
use crate::transaction::{Transaction, UTXO};

fn connection() -> sqlite::Connection {
    sqlite::Connection::open("simple-rust-coin.sqlite").unwrap()
}

pub fn get_frontier_block() -> Result<Block, ()> {
    let connection = connection();
    let statement = connection.prepare("SELECT * FROM blocks ORDER BY number DESC, timestamp ASC").unwrap();
    return_block_from_statement(statement)
}

fn return_block_from_statement(mut statement: Statement) -> Result<Block, ()> {
    match statement.next() {
        Ok(_) => {
            Ok(Block {
                number: statement.read::<i64, _>("number").unwrap() as u32,
                hash: statement.read::<String, _>("hash").unwrap(),
                prev_hash: statement.read::<String, _>("prev_hash").unwrap(),
                transactions: serde_json::from_str::<Vec<Transaction>>(&*statement.read::<String, _>("transactions").unwrap()).unwrap(),
                nonce: statement.read::<i64, _>("nonce").unwrap() as u64,
                timestamp: statement.read::<i64, _>("timestamp").unwrap() as u64,
                miner: statement.read::<String, _>("miner").unwrap()
            })
        }
        Err(_) => {
            Err(())
        }
    }
}

