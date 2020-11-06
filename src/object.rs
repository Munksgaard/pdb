use crate::ast::Ident;
use anyhow::Result;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Object {
    Int(i64),
    Bool(bool),
    Tuple(Vec<Object>),
    Unit,
    String(String),
    Record(Vec<(Ident, Object)>),
    Closure(Rc<dyn Fn(Object) -> Result<Object>>),
}

// Implement <BookFormat> == <Book> comparisons
impl PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        use Object::*;

        match (self, other) {
            (Int(i1), Int(i2)) => i1 == i2,
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Tuple(objs1), Tuple(objs2)) => {
                objs1.len() == objs2.len() && objs1.iter().eq(objs2.iter())
            }
            (Unit, Unit) => true,
            (String(s1), String(s2)) => s1 == s2,
            (Record(recs1), Record(recs2)) => {
                recs1.len() == recs2.len() && recs1.iter().eq(recs2.iter())
            }
            _ => false,
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Int(i) => write!(f, "{}", i),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Tuple(objs) => {
                let mut objs = objs.iter();
                write!(f, "(")?;

                if let Some(obj) = objs.next() {
                    write!(f, "{}", obj)?;
                }

                for obj in objs {
                    write!(f, ", {}", obj)?;
                }

                write!(f, ")")
            }
            Object::Unit => write!(f, "()"),
            Object::String(b) => write!(f, "{}", b),
            Object::Record(pairs) => {
                let mut pairs = pairs.iter();
                write!(f, "{{")?;

                if let Some((ident, obj)) = pairs.next() {
                    write!(f, "{} = {}", ident, obj)?;
                }

                for (ident, obj) in pairs {
                    write!(f, ", {} = {}", ident, obj)?;
                }

                write!(f, "}}")
            }
            Object::Closure(_) => write!(f, "<lambda>"),
        }
    }
}
