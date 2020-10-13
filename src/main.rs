use anyhow::Result;
use std::sync::mpsc::channel;
use std::thread;

fn main() -> Result<()> {
    println!("Hello! This is pdb");

    // Create a simple streaming channel
    let (tx, rx) = channel();

    // Spawn Db handler thread
    let handler = thread::spawn(|| pdb::db::start(rx));

    pdb::cli::start(tx)?;

    handler.join().expect("Something unexpected happened!")?;

    Ok(())
}
