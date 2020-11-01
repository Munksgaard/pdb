use crate::ast::Ident;

#[derive(Debug)]
pub struct NameSource {
    counter: i64,
}

impl NameSource {
    pub fn new() -> Self {
        NameSource { counter: 0 }
    }

    pub fn fresh(&mut self, name: &str) -> Ident {
        let i = self.counter;
        self.counter += 1;
        format!("{}_{}", name, i)
    }
}

impl Default for NameSource {
    fn default() -> Self {
        Self::new()
    }
}
