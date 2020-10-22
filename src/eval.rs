use crate::ast::*;
use crate::environment::Environment;
use crate::object::Object;
use anyhow::Result;

pub fn eval(env: &Environment, expr: Expr) -> Result<Object> {
    match expr {
        Expr::Int(i) => Ok(Object::Int(i)),
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
        Expr::Ident(ident) => env.lookup(&ident),
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
        Expr::Apply(e1, e2) => unimplemented!(),
        Expr::Lambda(ident, e) => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eval_int() {
        assert_eq!(
            "42",
            format!("{}", eval(&Environment::new(), Expr::Int(42)).unwrap())
        );
    }

    #[test]
    fn eval_bool() {
        assert_eq!(
            "true",
            format!("{}", eval(&Environment::new(), Expr::Bool(true)).unwrap())
        );
    }

    #[test]
    fn eval_tuple() {
        assert_eq!(
            "(false, 43)",
            format!(
                "{}",
                eval(
                    &Environment::new(),
                    Expr::Tuple(vec!(Expr::Bool(false), Expr::Int(43)))
                )
                .unwrap()
            ),
        );
    }

    #[test]
    fn eval_unit() {
        assert_eq!(
            "()",
            format!("{}", eval(&Environment::new(), Expr::Unit).unwrap())
        );
    }

    #[test]
    fn eval_record() {
        assert_eq!(
            "{x = false, y = 42}",
            format!(
                "{}",
                eval(
                    &Environment::new(),
                    Expr::Record(vec!(
                        (String::from("x"), Expr::Bool(false)),
                        (String::from("y"), Expr::Int(42))
                    ))
                )
                .unwrap()
            )
        );
    }
}
