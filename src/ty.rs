use crate::ast::*;

pub fn matches_type(expr: &Expr, ty: &Ty) -> bool {
    match (expr, ty) {
        (Expr::Int(_), Ty::Int) => true,
        (Expr::Bool(_), Ty::Bool) => true,
        (Expr::Tuple(exprs), Ty::Tuple(tys)) => exprs
            .iter()
            .zip(tys.iter())
            .all(|(x, y)| matches_type(x, y)),
        (Expr::Unit, Ty::Unit) => true,
        (Expr::String(_), Ty::String) => true,
        (Expr::Record(expr_pairs), Ty::Record(ty_pairs)) => expr_pairs
            .iter()
            .zip(ty_pairs.iter())
            .all(|((exprident, expr), (tyident, ty))| {
                exprident == tyident && matches_type(expr, ty)
            }),
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matches_type_int() {
        assert!(matches_type(&Expr::Int(42), &Ty::Int));
    }

    #[test]
    fn matches_type_int_bool_fails() {
        assert!(!matches_type(&Expr::Int(42), &Ty::Bool));
    }

    #[test]
    fn matches_type_bool() {
        assert!(matches_type(&Expr::Bool(true), &Ty::Bool));
    }

    #[test]
    fn matches_type_tuple() {
        assert!(matches_type(
            &Expr::Tuple(vec!(Expr::Bool(true), Expr::Int(42))),
            &Ty::Tuple(vec!(Ty::Bool, Ty::Int))
        ));
    }

    #[test]
    fn matches_type_unit() {
        assert!(matches_type(&Expr::Unit, &Ty::Unit));
    }

    #[test]
    fn matches_type_record() {
        assert!(matches_type(
            &Expr::Record(vec!(
                (String::from("x"), Expr::Bool(true)),
                (String::from("y"), Expr::Int(42))
            )),
            &Ty::Record(vec!(
                (String::from("x"), Ty::Bool),
                (String::from("y"), Ty::Int)
            ))
        ));
    }
}
