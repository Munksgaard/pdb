#[derive(Debug, PartialEq)]
pub enum Ty {
    Int,
    Bool,
}

#[derive(Debug, PartialEq)]
pub struct TableDefinition {
    pub ty: Ty,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Insert(Expr),
    Select,
}
