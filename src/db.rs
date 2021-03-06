use crate::ast::{Ident, Statement, TableDefinition};
use crate::environment::Environment;
use crate::eval::eval;
use crate::name_source::NameSource;
use crate::object::Object;
use crate::ty;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

type Tables = Vec<(Ident, TableDefinition, Vec<Object>)>;

struct Env {
    ty_env: ty::Env,
    env: Environment,
}

pub fn start(rx: Receiver<(Statement, Sender<Result<String>>)>) -> Result<()> {
    let mut tables: Tables = Vec::new();
    let mut env = Env {
        ty_env: HashMap::new(),
        env: Environment::new(),
    };

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
                    // infer type of expr and try to unify with def.ty
                    let ty = ty::infer(
                        &mut HashMap::new(),
                        &mut NameSource::new(),
                        &env.ty_env,
                        &expr,
                    )
                    .map_err(|e| anyhow!("{}", e))?;

                    if ty::unify(std::iter::once((ty.clone(), def.ty.clone()))).all(|x| x.is_ok()) {
                        let result = eval(&env.env, expr)?;
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
            Statement::Let(ident, expr) => {
                // infer type of expr and try to unify with def.ty
                let ty = ty::infer(
                    &mut HashMap::new(),
                    &mut NameSource::new(),
                    &env.ty_env,
                    &expr,
                )
                .map_err(|e| anyhow!("{}", e))?;

                let scheme = ty::generalize(&env.ty_env, ty.clone());

                env.ty_env.insert(ident.clone(), scheme);

                let obj = eval(&env.env, expr)?;

                env.env = env.env.insert(&ident, obj);

                tx.send(Ok(format!("{}: {}\n", ident, ty)))?;
            }
        }
    }
}
