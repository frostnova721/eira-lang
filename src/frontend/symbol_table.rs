use std::collections::HashMap;

use crate::frontend::{tapestry::Tapestry, weaves::Weave};

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

pub struct Symbol {
    pub name: String,
    pub weave: Weave,
    pub depth: usize,
    pub mutable: bool,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut scopes: Vec<HashMap<String, Symbol>> = vec![];
        scopes.push(HashMap::new());
        SymbolTable { scopes: scopes }
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, weave: Weave, mutable: bool) {
        let depth = self.scopes.len();

        if let Some(scope) = self.scopes.last_mut() {
            let symbol = Symbol {
                name: name.clone(),
                mutable: mutable,
                weave: weave,
                depth: depth,
            };
            scope.insert(name, symbol);
        } else {
            println!("No scopes???!!! Impossible!")
        }
    }

    pub fn resolve(&mut self, name: &String) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var);
            }
        }
        None
    }
}