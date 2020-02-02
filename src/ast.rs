pub type Ident = String;

#[derive(Debug, PartialEq)]
pub enum Ty {
    Int,
    Bool,
    Tuple(Vec<Ty>),
    Unit,
    Record(Vec<(Ident, Ty)>),
}

#[derive(Debug, PartialEq)]
pub struct TableDefinition {
    pub ty: Ty,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Tuple(Vec<Expr>),
    Unit,
    Record(Vec<(Ident, Expr)>),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Create(Ident, TableDefinition),
    Insert(Ident, Expr),
    Select(Ident),
}
