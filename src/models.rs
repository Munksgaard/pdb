#[derive(Debug, PartialEq)]
pub struct TyDef(pub Ty);

#[derive(Debug, PartialEq)]
pub struct TableDefinition {
    pub tydef: TyDef,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Insert(Expr),
    Select,
}
#[derive(Debug, PartialEq)]
pub enum Ty {
    Int,
    Bool,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
}
