use crate::ast::Ident;
use anyhow::Result;
use std::fmt;

pub enum Object {
    Int(i64),
    Bool(bool),
    Tuple(Vec<Object>),
    Unit,
    String(String),
    Record(Vec<(Ident, Object)>),
    Lambda(Box<dyn Fn(Object) -> Result<Object>>),
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        use Object::*;
        match self {
            Int(i) => Int(*i),
            Bool(b) => Bool(*b),
            Tuple(objs) => Tuple(objs.clone()),
            Unit => Unit,
            String(s) => String(s.clone()),
            Record(recs) => Record(recs.clone()),
            Lambda(_) => panic!("Cannot clone function objects"),
        }
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
            Object::Lambda(_) => write!(f, "<lambda>"),
        }
    }
}
