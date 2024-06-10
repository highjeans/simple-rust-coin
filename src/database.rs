use sqlite;
use sqlite::Statement;
use crate::block::Block;
use crate::transaction::{Transaction, UTXO};

fn connection() -> sqlite::Connection {
    sqlite::Connection::open("simple-rust-coin.sqlite").unwrap()
}

pub fn initialize_database() {
    let connection = connection();
    connection.execute("CREATE TABLE IF NOT EXISTS utxos (id integer PRIMARY KEY NOT NULL, amount real NOT NULL CHECK ( typeof(amount) = 'real' ), owner VARCHAR NOT NULL CHECK ( typeof(owner) = 'text' AND length(owner) = 64), used INTEGER NOT NULL DEFAULT FALSE);").unwrap();
    connection.execute("CREATE TABLE IF NOT EXISTS blocks (number integer NOT NULL CHECK ( typeof(number) = 'integer' AND number >= 0 ), hash VARCHAR PRIMARY KEY NOT NULL CHECK ( typeof(hash) = 'text' AND length(hash) = 128), prev_hash VARCHAR NOT NULL CHECK ( typeof(prev_hash) = 'text' AND length(prev_hash) = 128), transactions VARCHAR NOT NULL CHECK ( typeof(transactions) = 'text' ), nonce integer NOT NULL CHECK ( typeof(nonce) = 'integer' ), timestamp text NOT NULL CHECK ( timestamp IS date(timestamp, 'subsec')), miner text NOT NULL CHECK ( typeof(miner) = 'text' AND length(miner) = 64 ));").unwrap();
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

