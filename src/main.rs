use pdb;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello! This is pdb");
    println!("Feel free to type in commands");

    let stdin = io::stdin();
    let stdout = io::stdout();

    pdb::repl::start(&mut stdin.lock(), &mut stdout.lock())
        .unwrap_or_else(|e| panic!("An error occurred: {}", e));

    Ok(())
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let ty = parse::parse_tabledef("table Int")?;

//     println!("{:?}", ty);

//     let expr = parse::parse("insert 123");

//     println!("{:?}", expr);

//     Ok(())
// }
