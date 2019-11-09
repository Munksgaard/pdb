use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Object {
    Int(i64),
    Bool(bool),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Int(i) => write!(f, "{}", i),
            Object::Bool(b) => write!(f, "{}", b),
        }
    }
}
