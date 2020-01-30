use crate::ast::*;

pub fn unify(expr: &Expr, ty: &Ty) -> bool {
    match (expr, ty) {
        (Expr::Int(_), Ty::Int) => true,
        (Expr::Bool(_), Ty::Bool) => true,
        (Expr::Tuple(exprs), Ty::Tuple(tys)) => {
            exprs.iter().zip(tys.iter()).all(|(x, y)| unify(x, y))
        }
        (Expr::Unit, Ty::Unit) => true,
        (Expr::Record(expr_pairs), Ty::Record(ty_pairs)) => expr_pairs
            .iter()
            .zip(ty_pairs.iter())
            .all(|((exprident, expr), (tyident, ty))| exprident == tyident && unify(expr, ty)),
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unify_int() {
        assert!(unify(&Expr::Int(42), &Ty::Int));
    }

    #[test]
    fn unify_int_bool_fails() {
        assert!(!unify(&Expr::Int(42), &Ty::Bool));
    }

    #[test]
    fn unify_bool() {
        assert!(unify(&Expr::Bool(true), &Ty::Bool));
    }

    #[test]
    fn unify_tuple() {
        assert!(unify(
            &Expr::Tuple(vec!(Expr::Bool(true), Expr::Int(42))),
            &Ty::Tuple(vec!(Ty::Bool, Ty::Int))
        ));
    }

    #[test]
    fn unify_unit() {
        assert!(unify(&Expr::Unit, &Ty::Unit));
    }

    #[test]
    fn unify_record() {
        assert!(unify(
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
