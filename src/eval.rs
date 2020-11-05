use crate::ast::*;
use crate::environment::Environment;
use crate::object::Object;
use anyhow::anyhow;
use anyhow::Result;
use std::rc::Rc;

#[cfg(test)]
mod test;

pub fn eval_atom(env: &Environment, atom: Atom) -> Result<Object> {
    match atom {
        Atom::Int(i) => {
            let res = Object::Int(i);
            Ok(res)
        }
        Atom::Bool(b) => Ok(Object::Bool(b)),
        Atom::Unit => Ok(Object::Unit),
        Atom::String(b) => Ok(Object::String(b)),
        Atom::Ident(ident) => env.lookup(&ident).map(|x| x.clone()),
    }
}

pub fn eval(env: &Environment, expr: Expr) -> Result<Object> {
    match expr {
        Expr::Atom(atom) => Ok(eval_atom(env, atom)?),
        Expr::Tuple(exprs) => Ok(Object::Tuple(
            exprs
                .into_iter()
                .map(|e| eval(env, e))
                .collect::<Result<Vec<_>>>()?,
        )),
        Expr::Record(xs) => {
            let res = xs
                .into_iter()
                .map(|(ident, obj)| Ok((ident, eval(env, obj)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(Object::Record(res))
        }
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
        Expr::Case(expr, matches) => {
            let obj = eval(env, *expr)?;

            for (pat, e) in matches {
                // See if obj matches obj, returning an updated environment
                if let Some(env) = match_pat(env, &pat, &obj) {
                    return eval(&env, e);
                }
            }

            Err(anyhow!("No match found for case!"))
        }
    }
}

fn match_pat(env: &Environment, pat: &Pattern, obj: &Object) -> Option<Environment> {
    match (pat, obj) {
        (Pattern::Ident(ident), _) => Some(env.insert(&ident, obj.clone())),
        (Pattern::Tuple(pats), Object::Tuple(objs)) => {
            // I assume that they match, since we've already type checked
            let mut env = env.clone();
            for (pat, obj) in pats.iter().zip(objs.iter()) {
                env = match_pat(&env, pat, &obj)?
            }
            Some(env)
        }
        _ => None,
    }
}
