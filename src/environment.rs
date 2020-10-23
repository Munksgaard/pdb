use crate::object::*;
use anyhow::{anyhow, Result};
use std::sync::Arc;

#[derive(Clone)]
pub enum Environment {
    Node(String, Object, Arc<Environment>),
    Empty,
}

impl Environment {
    pub fn new() -> Environment {
        Environment::Empty
    }

    pub fn lookup(&self, ident: &str) -> Result<&Object> {
        match self {
            Environment::Node(s, obj, inner) => {
                if s == ident {
                    Ok(obj)
                } else {
                    inner.lookup(ident)
                }
            }
            Environment::Empty => Err(anyhow!("Identifier {} not found", ident)),
        }
    }

    pub fn insert(&self, ident: &str, obj: Object) -> Environment {
        Environment::Node(ident.to_string(), obj, Arc::new(self.clone()))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
