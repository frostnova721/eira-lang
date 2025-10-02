use std::collections::HashMap;

use crate::{frontend::{weaves::Weave}};

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub weave: Weave,
    pub depth: usize,
    pub mutable: bool,
    pub slot_idx: usize,
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

    pub fn define(&mut self, name: String, weave: Weave, mutable: bool, slot_idx: usize) -> Option<Symbol> {
        let depth = self.scopes.len() - 1;

        if let Some(scope) = self.scopes.last_mut() {
            let symbol = Symbol {
                name: name.clone(),
                mutable: mutable,
                weave: weave,
                depth: depth,
                slot_idx: slot_idx
            };
            scope.insert(name, symbol.clone());
            return Some(symbol);
        } else {
            // This branch is literally impossible!
            println!("No scopes???!!! Impossible!");
            None
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

    pub fn get_current_scope_size(&self) -> usize {
        self.scopes.last().unwrap().len()
    }

    pub fn get_depth(&self) -> usize {
        self.scopes.len() - 1
    }
}