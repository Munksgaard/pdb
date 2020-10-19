use serde_derive::{Deserialize, Serialize};

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
