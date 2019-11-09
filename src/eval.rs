use crate::ast::*;
use crate::object::Object;

pub fn eval(expr: Expr) -> Object {
    match expr {
        Expr::Int(i) => Object::Int(i),
        Expr::Bool(b) => Object::Bool(b),
    }
}
