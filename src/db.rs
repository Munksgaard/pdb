use crate::ast::{Ident, Statement, TableDefinition, Ty};
use crate::environment::Environment;
use crate::eval::eval;
use crate::name_source::NameSource;
use crate::object::Object;
use crate::ty;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

type Tables = Vec<(Ident, TableDefinition, Vec<Object>)>;

pub struct Env {
    ty_env: ty::Env,
    env: Environment,
    tables: Tables,
    constructors: HashMap<Ident, (Vec<Ty>, Ident)>,
}

pub fn eval_stm(env: &mut Env, stm: Statement) -> Result<String> {
    match stm {
        Statement::Create(ident, def) => {
            env.tables.push((ident, def, Vec::new()));
            Ok(String::from("Created\n"))
        }
        Statement::Insert(ident, expr) => {
            if let Some((_, def, objs)) = env
                .tables
                .iter_mut()
                .find(|(ident2, _, _)| ident2 == &ident)
            {
                // infer type of expr and try to unify with def.ty
                let ty = ty::infer(
                    &mut HashMap::new(),
                    &mut NameSource::new(),
                    &env.ty_env,
                    &expr,
                )
                .map_err(|e| anyhow!("{}", e))?;

                if ty::unify(std::iter::once((ty, def.ty.clone()))).all(|x| x.is_ok()) {
                    let result = eval(&env.env, expr)?;
                    objs.push(result);
                    Ok(String::from("Inserted 1\n"))
                } else {
                    Err(anyhow!(
                        "Could not insert {:?} into table {:?} with definition {:?}\n",
                        expr,
                        ident,
                        &def.ty
                    ))
                }
            } else {
                Err(anyhow!("No such table\n"))
            }
        }
        Statement::Select(ident) => {
            if let Some((_, _, objs)) = env.tables.iter().find(|(ident2, _, _)| ident2 == &ident) {
                Ok(format!("{:?}\n", objs))
            } else {
                Err(anyhow!("No such table\n"))
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
            .map_err(|e| anyhow!("{}", e))
            .unwrap();

            let scheme = ty::generalize(&env.ty_env, ty.clone());

            env.ty_env.insert(ident.clone(), scheme);

            let obj = eval(&env.env, expr)?;

            env.env = env.env.insert(&ident, obj);

            Ok(format!("{}: {}\n", ident, ty))
        }
        Statement::Union(name, _args, variants) => {
            for (variant_name, tyargs) in variants {
                env.constructors
                    .insert(variant_name.clone(), (tyargs.clone(), name.clone()));
            }
            Ok(String::from("Ok\n"))
        }
    }
}

pub fn start(rx: Receiver<(Statement, Sender<Result<String>>)>) -> Result<()> {
    let mut env = Env {
        ty_env: HashMap::new(),
        env: Environment::new(),
        tables: Vec::new(),
        constructors: HashMap::new(),
    };

    loop {
        let (stm, tx) = match rx.recv() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Shutting down db (cause: {})", e);
                return Ok(());
            }
        };

        let response = eval_stm(&mut env, stm);

        tx.send(response)
            .context("Ack channel prematurely closed")?;
    }
}
