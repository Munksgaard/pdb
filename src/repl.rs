use crate::ast::{Ident, Statement, TableDefinition};
use crate::eval::eval;
use crate::object::Object;
use crate::parse::parse;
use crate::ty::matches_type;
use anyhow::{anyhow, Error, Result};
use rustyline::error::ReadlineError;
use rustyline::Editor;

const PROMPT: &str = ">> ";

type Tables = Vec<(Ident, TableDefinition, Vec<Object>)>;

fn parse_helper(tables: &mut Tables, rl: &mut Editor<()>, line: &str) {
    match parse(&line) {
        Ok(ast) => match ast {
            Statement::Create(ident, def) => {
                tables.push((ident, def, Vec::new()));
                println!("Created\n");
            }
            Statement::Insert(ident, expr) => {
                if let Some((_, def, objs)) =
                    tables.iter_mut().find(|(ident2, _, _)| ident2 == &ident)
                {
                    if matches_type(&expr, &def.ty) {
                        let result = eval(expr);
                        objs.push(result);
                        println!("Inserted 1\n");
                    } else {
                        println!(
                            "Could not insert {:?} into table {:?} with definition {:?}\n",
                            expr, ident, &def.ty
                        );
                    }
                } else {
                    println!("No such table\n");
                }
            }
            Statement::Select(ident) => {
                if let Some((_, _, objs)) = tables.iter().find(|(ident2, _, _)| ident2 == &ident) {
                    println!("{:?}\n", objs);
                } else {
                    println!("No such table\n");
                }
            }
        },
        Err(e) => {
            println!("No parse: {}\n", e);
        }
    };
    rl.add_history_entry(line);
}

pub fn start() -> Result<()> {
    let mut tables: Tables = Vec::new();

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => parse_helper(&mut tables, &mut rl, &line),
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
