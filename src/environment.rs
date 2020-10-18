use crate::object::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Environment {
    inner: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            inner: HashMap::new(),
        }
    }

    pub fn lookup(&self, ident: &str) -> Result<Object> {
        self.inner
            .get(ident)
            .cloned()
            .ok_or_else(|| anyhow!("Identifier {} not found", ident))
    }

    pub fn insert(mut self, ident: &str, e: Object) -> Self {
        self.inner.insert(ident.to_string(), e);
        self
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
