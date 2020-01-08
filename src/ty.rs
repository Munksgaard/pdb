use crate::ast::*;

pub fn unify(expr: &Expr, ty: &Ty) -> bool {
    match (expr, ty) {
        (Expr::Int(_), Ty::Int) => true,
        (Expr::Bool(_), Ty::Bool) => true,
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
    }
}
