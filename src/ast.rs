use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
mod test;

pub type Ident = String;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Ty {
    Int,
    Bool,
    Tuple(Vec<Ty>),
    Unit,
    String,
    Record(Vec<(Ident, Ty)>),
    Var(Ident),
    Fun(Box<Ty>, Box<Ty>),
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int => write!(f, "Int"),
            Ty::Bool => write!(f, "Bool"),
            Ty::Tuple(tys) => {
                write!(f, "(")?;
                for (i, ty) in tys.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }

                write!(f, ")")
            }
            Ty::Unit => write!(f, "()"),
            Ty::String => write!(f, "String"),
            Ty::Record(recs) => {
                write!(f, "{{ ")?;
                for (i, (ident, ty)) in recs.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", ident, ty)?;
                }

                write!(f, " }}")
            }
            Ty::Var(ident) => write!(f, "{}", ident),
            Ty::Fun(lhs, rhs) => write!(f, "({} -> {})", lhs, rhs), // TODO: Handle parenthesis
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TableDefinition {
    pub ty: Ty,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Tuple(Vec<Expr>),
    Unit,
    String(String),
    Record(Vec<(Ident, Expr)>),
    Ident(Ident),
    Let(Vec<(Ident, Expr)>, Box<Expr>),
    Apply(Box<Expr>, Box<Expr>),
    Lambda(Ident, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Tuple(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }

                write!(f, ")")
            }
            Expr::Unit => write!(f, "()"),
            Expr::String(s) => write!(f, "{:?}", s),
            Expr::Ident(ident) => write!(f, "{}", ident),
            Expr::Record(recs) => {
                write!(f, "{{ ")?;
                for (i, (ident, expr)) in recs.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", ident, expr)?;
                }

                write!(f, " }}")
            }
            Expr::Let(binds, e) => {
                for (ident, expr) in binds.iter() {
                    write!(f, "let {} = {} in ", ident, expr)?;
                }

                write!(f, "{} end", e)
            }
            Expr::Apply(e1, e2) => write!(f, "({} {})", e1, e2), // TODO: Handle parenthesis
            Expr::Lambda(ident, expr) => write!(f, "lambda {} -> {}", ident, expr),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Create(Ident, TableDefinition),
    Insert(Ident, Expr),
    Select(Ident),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Create(ident, def) => write!(f, "create table {} {}", ident, def.ty),
            Statement::Insert(ident, expr) => write!(f, "insert {} into {}", expr, ident),
            Statement::Select(ident) => write!(f, "select from {}", ident),
        }
    }
}

pub type Statements = Vec<Statement>;
