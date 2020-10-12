use anyhow::Result;

fn main() -> Result<()> {
    println!("Hello! This is pdb");
    println!("Feel free to type in commands");

    pdb::repl::start().unwrap_or_else(|e| panic!("An error occurred: {}", e));

    Ok(())
}
