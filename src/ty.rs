use crate::ast::*;

pub fn unify(expr: &Expr, ty: &Ty) -> bool {
    match (expr, ty) {
        (Expr::Int(_), Ty::Int) => true,
        (Expr::Bool(_), Ty::Bool) => true,
        (Expr::Tuple(exprs), Ty::Tuple(tys)) => {
            exprs.iter().zip(tys.iter()).all(|(x, y)| unify(x, y))
        }
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unify_test() {
        assert!(unify(&Expr::Int(42), &Ty::Int));
        assert!(!unify(&Expr::Int(42), &Ty::Bool));
        assert!(unify(&Expr::Bool(true), &Ty::Bool));
        assert!(unify(
            &Expr::Tuple(vec!(Expr::Bool(true), Expr::Int(42))),
            &Ty::Tuple(vec!(Ty::Bool, Ty::Int))
        ));
    }
}
