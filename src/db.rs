use crate::ast::{Ident, Statement, TableDefinition};
use crate::eval::eval;
use crate::object::Object;
use crate::ty::matches_type;
use anyhow::Result;
use std::sync::mpsc::Receiver;

type Tables = Vec<(Ident, TableDefinition, Vec<Object>)>;

pub fn start(rx: Receiver<Statement>) -> Result<()> {
    let mut tables: Tables = Vec::new();

    loop {
        let stm = match rx.recv() {
            Ok(stm) => stm,
            Err(e) => {
                println!("Shutting down db (cause: {})", e);
                return Ok(());
            }
        };

        match stm {
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
        }
    }
}
