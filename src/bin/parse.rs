use anyhow::{anyhow, Error, Result};
use pdb::parse::*;
use pest::Parser as _;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use structopt::StructOpt;

const PROMPT: &str = ">> ";

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "type")]
    Type,

    #[structopt(name = "expr")]
    Expr,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "pdbcli", about = "A simple client for pdb.")]
struct Args {
    #[structopt(subcommand)]
    command: Command,
}

#[paw::main]
fn main(args: Args) -> Result<()> {
    match args.command {
        Command::Type => start_type(),
        Command::Expr => start_expr(),
    }
}

fn start_expr() -> Result<()> {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                let parsed = Parser::parse(Rule::expr, &line)?.next().unwrap();
                println!("{:?}", parsed);
                println!("{:?}", parse_exprs(parsed.into_inner()));
                rl.add_history_entry(line);
            }
            Err(ReadlineError::Interrupted) => break Err(anyhow!("unimplemented")),
            Err(ReadlineError::Eof) => return Ok(()),
            Err(err) => break Err(Error::new(err)),
        }
    }
}

fn start_type() -> Result<()> {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                let parsed = Parser::parse(Rule::ty, &line)?.next().unwrap();
                println!("{:?}", parsed);
                println!("{:?}", parse_ty(parsed.into_inner().next().unwrap()));
                rl.add_history_entry(line);
            }
            Err(ReadlineError::Interrupted) => break Err(anyhow!("unimplemented")),
            Err(ReadlineError::Eof) => return Ok(()),
            Err(err) => break Err(Error::new(err)),
        }
    }
}
