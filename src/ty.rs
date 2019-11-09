use crate::ast::*;

pub fn unify(expr: &Expr, tabledef: &TableDefinition) -> bool {
    match (expr, &tabledef.tydef) {
        (Expr::Int(_), TyDef(Ty::Int)) => true,
        (Expr::Bool(_), TyDef(Ty::Bool)) => true,
        _ => false,
    }
}
