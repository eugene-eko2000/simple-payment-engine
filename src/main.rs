use std::time::Instant;

use anyhow::Result;
use clap::Parser;

use crate::transaction::Transaction;

mod client;
mod engine;
mod transaction;

#[derive(Debug, Parser)]
pub struct Args {
    /// Input CSV file containing transactions
    #[clap(value_parser)]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut engine = engine::Engine::new();

    let mut reader = csv::Reader::from_path(args.input)?;
    let mut counter = 0u64;
    let start = Instant::now();
    for rec in reader.records() {
        let record = rec?;
        let transaction = record.deserialize(None);
        if let Err(err) = transaction {
            eprintln!("Failed to deserialize transaction: {}", err);
            continue;
        }
        let transaction: Transaction = transaction.unwrap();
        if let Err(err) = engine.execute(transaction) {
            eprintln!("Failed to execute transaction: {:?}", err);
        }
        counter += 1;
        if counter % 1000000 == 0 {
            eprintln!("Processed {} transactions...", counter);
        }
    }
    let duration = start.elapsed();
    eprintln!("Processed {} transactions in {:?}", counter, duration);

    engine.print_client_report();

    Ok(())
}
