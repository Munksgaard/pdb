use crate::ast::{Ident, Statement, TableDefinition};
use crate::environment::Environment;
use crate::eval::eval;
use crate::object::Object;
use crate::ty::matches_type;
use anyhow::{anyhow, Context, Result};
use std::sync::mpsc::{Receiver, Sender};

type Tables = Vec<(Ident, TableDefinition, Vec<Object>)>;

pub fn start(rx: Receiver<(Statement, Sender<Result<String>>)>) -> Result<()> {
    let mut tables: Tables = Vec::new();

    loop {
        let (stm, tx) = match rx.recv() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Shutting down db (cause: {})", e);
                return Ok(());
            }
        };

        match stm {
            Statement::Create(ident, def) => {
                tables.push((ident, def, Vec::new()));
                tx.send(Ok(String::from("Created\n")))
                    .context("Ack channel prematurely closed")?;
            }
            Statement::Insert(ident, expr) => {
                if let Some((_, def, objs)) =
                    tables.iter_mut().find(|(ident2, _, _)| ident2 == &ident)
                {
                    if matches_type(&expr, &def.ty) {
                        let result = eval(&Environment::new(), expr)?;
                        objs.push(result);
                        tx.send(Ok(String::from("Inserted 1\n")))
                            .context("Ack channel prematurely closed")?;
                    } else {
                        tx.send(Err(anyhow!(
                            "Could not insert {:?} into table {:?} with definition {:?}\n",
                            expr,
                            ident,
                            &def.ty
                        )))
                        .context("Ack channel prematurely closed")?;
                    }
                } else {
                    tx.send(Err(anyhow!("No such table\n")))
                        .context("Ack channel prematurely closed")?;
                }
            }
            Statement::Select(ident) => {
                if let Some((_, _, objs)) = tables.iter().find(|(ident2, _, _)| ident2 == &ident) {
                    tx.send(Ok(format!("{:?}\n", objs)))?;
                } else {
                    tx.send(Err(anyhow!("No such table\n")))
                        .context("Ack channel prematurely closed")?;
                }
            }
        }
    }
}
