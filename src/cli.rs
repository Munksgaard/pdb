use crate::ast::Statement;
use crate::parse::parse;
use anyhow::{anyhow, Error, Result};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::sync::mpsc::{channel, Sender};

const PROMPT: &str = ">> ";

pub fn start(tx: Sender<(Statement, Sender<Result<String>>)>) -> Result<()> {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                match parse(&line) {
                    Ok(ast) => {
                        let (tx2, rx2) = channel();

                        tx.send((ast, tx2)).expect("unimplemented");

                        match rx2.recv()? {
                            Ok(msg) => println!("OK: {}", msg),
                            Err(e) => println!("Err: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("No parse: {}\n", e);
                    }
                };
                rl.add_history_entry(line);
            }
            Err(ReadlineError::Interrupted) => {
                break Err(anyhow!("unimplemented"));
            }
            Err(ReadlineError::Eof) => {
                return Ok(());
            }
            Err(err) => break Err(Error::new(err)),
        }
    }
}
