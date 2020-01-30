use crate::ast::*;
use crate::object::Object;

pub fn eval(expr: Expr) -> Object {
    match expr {
        Expr::Int(i) => Object::Int(i),
        Expr::Bool(b) => Object::Bool(b),
        Expr::Tuple(exprs) => Object::Tuple(exprs.clone().into_iter().map(eval).collect()),
        Expr::Unit => Object::Unit,
        Expr::Record(xs) => Object::Record(
            xs.clone()
                .into_iter()
                .map(|(ident, obj)| (ident, eval(obj)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eval_int() {
        assert_eq!(eval(Expr::Int(42)), Object::Int(42));
    }

    #[test]
    fn eval_bool() {
        assert_eq!(eval(Expr::Bool(true)), Object::Bool(true));
    }

    #[test]
    fn eval_tuple() {
        assert_eq!(
            eval(Expr::Tuple(vec!(Expr::Bool(false), Expr::Int(43)))),
            Object::Tuple(vec!(Object::Bool(false), Object::Int(43)))
        );
    }

    #[test]
    fn eval_unit() {
        assert_eq!(eval(Expr::Unit), Object::Unit);
    }

    #[test]
    fn eval_record() {
        assert_eq!(
            eval(Expr::Record(vec!(
                (String::from("x"), Expr::Bool(false)),
                (String::from("y"), Expr::Int(42))
            ))),
            Object::Record(vec!(
                (String::from("x"), Object::Bool(false)),
                (String::from("y"), Object::Int(42))
            ))
        );
    }
}
