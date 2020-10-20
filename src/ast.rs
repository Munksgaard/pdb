use serde_derive::{Deserialize, Serialize};
use std::fmt;

pub type Ident = String;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
            Ty::Fun(lhs, rhs) => write!(f, "{} -> {}", lhs, rhs),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TableDefinition {
    pub ty: Ty,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Create(Ident, TableDefinition),
    Insert(Ident, Expr),
    Select(Ident),
}

pub type Statements = Vec<Statement>;
