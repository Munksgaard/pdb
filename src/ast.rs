#[derive(Debug, PartialEq)]
pub enum Ty {
    Int,
    Bool,
    Tuple(Vec<Ty>),
}

#[derive(Debug, PartialEq)]
pub struct TableDefinition {
    pub ty: Ty,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Tuple(Vec<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Insert(Expr),
    Select,
}
