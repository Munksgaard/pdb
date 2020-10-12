use anyhow::Result;
use pdb;
use std::io;

fn main() -> Result<()> {
    println!("Hello! This is pdb");
    println!("Feel free to type in commands");

    let stdin = io::stdin();
    let stdout = io::stdout();

    pdb::repl::start(&mut stdin.lock(), &mut stdout.lock())
        .unwrap_or_else(|e| panic!("An error occurred: {}", e));

    Ok(())
}
