use sqlite;
use sqlite::Statement;
use crate::block::Block;
use crate::transaction::{Transaction, UTXO};

const DEFAULT_DIFFICULTY: u32 = 25;

fn connection() -> sqlite::Connection {
    sqlite::Connection::open("simple-rust-coin.sqlite").unwrap()
}

pub fn initialize_database() {
    let connection = connection();
    connection.execute("CREATE TABLE IF NOT EXISTS utxos (id integer PRIMARY KEY NOT NULL, amount real NOT NULL CHECK ( typeof(amount) = 'real' ), owner VARCHAR NOT NULL CHECK ( typeof(owner) = 'text' AND length(owner) = 64), used INTEGER NOT NULL DEFAULT FALSE);").unwrap();
    connection.execute("CREATE TABLE IF NOT EXISTS blocks (number integer NOT NULL CHECK ( typeof(number) = 'integer' AND number >= 0 ), hash VARCHAR PRIMARY KEY NOT NULL CHECK ( typeof(hash) = 'text' AND length(hash) = 128), prev_hash VARCHAR NOT NULL CHECK ( typeof(prev_hash) = 'text' AND length(prev_hash) = 128), transactions VARCHAR NOT NULL CHECK ( typeof(transactions) = 'text' ), nonce integer NOT NULL CHECK ( typeof(nonce) = 'integer' ), timestamp text NOT NULL CHECK ( timestamp IS date(timestamp, 'subsec')), miner text NOT NULL CHECK ( typeof(miner) = 'text' AND length(miner) = 64 ));").unwrap();
}

pub fn get_mining_difficulty() -> u32 { // the difficulty is the number of leading 0s a block's hash should have
    match get_frontier_block() {
        Ok(block) => {
            let curr_timestamp = block.timestamp;
            match get_block_by_hash(block.prev_hash) {
                Ok(prev_block) => {
                    // try to get an average of a block every 5 minutes
                    let time_diff = curr_timestamp - prev_block.timestamp;
                    let default_difficulty_change: i64 = if time_diff == 0 {0} else if time_diff < 0 { -1 * (time_diff + 1).ilog2() as i64 } else { ((time_diff * -1) + 1).ilog2() as i64 };
                    if default_difficulty_change <= -1 * DEFAULT_DIFFICULTY as i64 { // ensure DEFAULT_DIFFICULTY + default_difficulty_change is positive and >= 1
                        return 1;
                    }
                    if default_difficulty_change as u32 + DEFAULT_DIFFICULTY > 512 { // ensure difficulty doesnt go past max number of bits
                        return 512;
                    }
                    return DEFAULT_DIFFICULTY + default_difficulty_change as u32;
                }
                Err(_) => {DEFAULT_DIFFICULTY} // 25 0s before the hash takes about 5 minutes on my laptop, good starting point ig
            }
        }
        Err(_) => {DEFAULT_DIFFICULTY} // 25 0s before the hash takes about 5 minutes on my laptop, good starting point ig
    }
}

pub fn get_frontier_block() -> Result<Block, ()> {
    let connection = connection();
    let statement = connection.prepare("SELECT * FROM blocks ORDER BY number DESC, timestamp ASC").unwrap();
    return_block_from_statement(statement)
}

pub fn get_block_by_hash(hash: String) -> Result<Block, ()>{
    let connection = connection();
    let mut statement = connection.prepare("SELECT * FROM blocks WHERE hash = ?").unwrap();
    statement.bind((1, hash.as_str())).unwrap();
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

pub fn get_valid_owner_utxos(owner: String) -> Vec<UTXO> {
    let connection = connection();
    let mut statement = connection.prepare("SELECT amount, owner FROM utxos WHERE owner = ? AND used = FALSE;").unwrap();
    statement.bind((1, owner.clone().as_str())).unwrap();
    let mut utxos = Vec::<UTXO>::new();
    while let Ok(_) = statement.next() {
        utxos.push(UTXO {
            amount: statement.read::<f64, _>("amount").unwrap(),
            owner: statement.read::<String, _>("owner").unwrap()
        })
    }
    utxos
}

pub fn insert_block(block: Block) -> sqlite::Result<()> {
    let connection = connection();
    let transactions_str = serde_json::to_string(&block.transactions).unwrap();
    let query = format!("INSERT INTO blocks (number, hash, prev_hash, transactions, nonce, timestamp, miner) VALUES ({}, '{}', '{}', {}, {}, {}, '{}'", block.number, block.hash.clone(), block.prev_hash.clone(), transactions_str, block.nonce, block.timestamp, block.miner.clone());
    connection.execute(query)?;

    // Add all new UTXOs generated from the transactions and mark input UTXOs as used
    for transaction in block.transactions {
        for utxo in transaction.input_utxos {
            let mut statement = connection.prepare("UPDATE utxos SET used = TRUE WHERE id = (SELECT min(id) FROM utxos WHERE amount = ? AND owner = ? AND used = FALSE);")?;
            statement.bind::<&[(i32, sqlite::Value)]>(&[
                (1, utxo.amount.try_into().unwrap()),
                (2, utxo.owner.clone().try_into().unwrap())
            ])?;
            statement.next()?;
        }

        for utxo in [transaction.output_utxos.0, transaction.output_utxos.1] {
            let mut statement = connection.prepare("INSERT INTO utxos (amount, owner) VALUES (?, ?)")?;
            statement.bind::<&[(i32, sqlite::Value)]>(&[
                (1, utxo.amount.try_into().unwrap()),
                (2, utxo.owner.clone().try_into().unwrap())
            ])?;
            statement.next()?;
        }
    }
    Ok(())
}

pub fn get_all_utxos() -> Vec<UTXO> {
    let connection = connection();
    let mut statement = connection.prepare("SELECT amount, owner FROM utxos WHERE used = FALSE;").unwrap();
    let mut utxos = Vec::<UTXO>::new();
    while let Ok(_) = statement.next() {
        utxos.push(UTXO {
            amount: statement.read::<f64, _>("amount").unwrap(),
            owner: statement.read::<String, _>("owner").unwrap()
        })
    }
    utxos
}
