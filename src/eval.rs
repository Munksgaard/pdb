use crate::ast::*;
use crate::environment::Environment;
use crate::object::Object;
use anyhow::Result;
use std::rc::Rc;

#[cfg(test)]
mod test;

pub fn eval(env: &Environment, expr: Expr) -> Result<Object> {
    match expr {
        Expr::Int(i) => {
            let res = Object::Int(i);
            Ok(res)
        }
        Expr::Bool(b) => Ok(Object::Bool(b)),
        Expr::Tuple(exprs) => Ok(Object::Tuple(
            exprs
                .into_iter()
                .map(|e| eval(env, e))
                .collect::<Result<Vec<_>>>()?,
        )),
        Expr::Unit => Ok(Object::Unit),
        Expr::String(b) => Ok(Object::String(b)),
        Expr::Record(xs) => {
            let res = xs
                .into_iter()
                .map(|(ident, obj)| Ok((ident, eval(env, obj)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(Object::Record(res))
        }
        Expr::Ident(ident) => env.lookup(&ident).map(|x| x.clone()),
        Expr::Let(binds, e) => {
            let env_ = binds
                .into_iter()
                .try_fold::<Environment, _, Result<Environment>>(
                    env.clone(),
                    |env, (ident, e_inner)| {
                        let obj = eval(&env, e_inner)?;
                        Ok(env.insert(&ident, obj))
                    },
                )?;
            eval(&env_, *e)
        }
        Expr::Apply(e1, e2) => {
            let obj = eval(env, *e2)?;
            match eval(env, *e1)? {
                Object::Closure(f) => f(obj),
                other => unreachable!("{}", other),
            }
        }
        Expr::Lambda(ident, e) => {
            let env = env.clone();
            Ok(Object::Closure(Rc::new(move |obj| {
                eval(&env.insert(&ident, obj), *e.clone())
            })))
        }
    }
}
