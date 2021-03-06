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
