use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    compiler::{types::Visibility, weaves::Weave},
    values::{sign::SignInfo, spell::SpellInfo},
};

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    exported_symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable { mutable: bool },
    Spell(SpellInfo),
    Sign(SignInfo),
}

impl SymbolKind {
    pub fn get_spell_info(&self) -> Option<SpellInfo> {
        match self {
            Self::Spell(i) => Some(i.clone()),
            _ => None,
        }
    }

    pub fn get_sign_info(&self) -> Option<SignInfo> {
        match self {
            Self::Sign(i) => Some(i.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub weave: Weave,
    pub depth: usize,
    pub kind: RefCell<SymbolKind>,
    pub slot_idx: usize,
    pub parent: Option<Rc<Symbol>>,
    pub visibility: Visibility,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut scopes: Vec<HashMap<String, Symbol>> = vec![];
        scopes.push(HashMap::new());
        SymbolTable {
            scopes: scopes,
            exported_symbols: vec![],
        }
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn modify_symbol(&mut self, symbol: Symbol) {
        self.scopes[symbol.depth].insert(symbol.name.clone(), symbol);
    }

    pub fn define_variable(
        &mut self,
        name: String,
        weave: Weave,
        mutable: bool,
        slot_idx: usize,
        parent: Option<Rc<Symbol>>,
        visibility: Visibility,
    ) -> Option<Symbol> {
        self.add_symbol(
            name,
            weave,
            SymbolKind::Variable { mutable },
            parent,
            slot_idx,
            visibility,
        )
    }

    pub fn define_spell(
        &mut self,
        name: String,
        weave: Weave,
        info: SpellInfo,
        slot_idx: usize,
        parent: Option<Rc<Symbol>>,
        visibility: Visibility,
    ) -> Option<Symbol> {
        let kind = SymbolKind::Spell(info);
        self.add_symbol(name, weave, kind, parent, slot_idx, visibility)
    }

    pub fn define_sign(
        &mut self,
        name: String,
        weave: Weave,
        info: SignInfo,
        parent: Option<Rc<Symbol>>,
        slot_idx: usize,
        visibility: Visibility,
    ) -> Option<Symbol> {
        let kind = SymbolKind::Sign(info);
        self.add_symbol(name, weave, kind, parent, slot_idx, visibility)
    }

    pub fn add_symbol(
        &mut self,
        name: String,
        weave: Weave,
        kind: SymbolKind,
        parent: Option<Rc<Symbol>>,
        slot_idx: usize,
        visibility: Visibility,
    ) -> Option<Symbol> {
        let depth = self.scopes.len() - 1;

        // the necessary conditions to be eligible for exports
        if depth == 0 && visibility == Visibility::Public {
            self.exported_symbols.push(name.clone());
        }

        if let Some(scope) = self.scopes.last_mut() {
            let symbol = Symbol {
                name: name.clone(),
                weave: weave,
                depth: depth,
                parent: parent,
                kind: RefCell::new(kind),
                slot_idx: slot_idx,
                visibility: visibility.clone(),
            };

            scope.insert(name, symbol.clone());

            return Some(symbol);
        } else {
            // This branch is literally impossible!
            println!("No scopes???!!! Impossible!");
            None
        }
    }

    /// Adds the symbol to ST without adding to export table
    pub fn import_symbol(
        &mut self,
        name: String,
        weave: Weave,
        kind: SymbolKind,
        parent: Option<Rc<Symbol>>,
        slot_idx: usize,
        visibility: Visibility,
    ) -> Option<Symbol> {
        let depth = self.scopes.len() - 1;

        if let Some(scope) = self.scopes.last_mut() {
            let symbol = Symbol {
                name: name.clone(),
                weave: weave,
                depth: depth,
                parent: parent,
                kind: RefCell::new(kind),
                slot_idx: slot_idx,
                visibility: visibility.clone(),
            };

            scope.insert(name, symbol.clone());

            return Some(symbol);
        } else {
            None
        }
    }

    pub fn resolve(&self, name: &String) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var);
            }
        }
        None
    }

    pub fn resolve_in_current_scope(&self, name: &String) -> Option<&Symbol> {
        self.scopes.last()?.get(name)
    }

    pub fn get_current_scope_size(&self) -> usize {
        self.scopes.last().unwrap().len()
    }

    pub fn get_depth(&self) -> usize {
        self.scopes.len() - 1
    }

    pub fn get_exports(&self) -> HashMap<String, Symbol> {
        let mut exports: HashMap<String, Symbol> = HashMap::new();
        let scope = self.scopes.first().unwrap();
        for name in &self.exported_symbols {
            if let Some(symbol) = scope.get(name) {
                exports.insert(name.clone(), symbol.clone());
            }
        }
        exports
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for (i, scope) in self.scopes.iter().enumerate() {
            result.push_str(&format!("Scope {}:\n", i));
            for (name, symbol) in scope {
                result.push_str(&format!("  {}: {:?}\n", name, symbol));
            }
        }
        write!(f, "{}", result)
    }
}
