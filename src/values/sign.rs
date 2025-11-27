use std::collections::HashMap;

use crate::{SpellObject, Value};

/// Represents a Sign (or struct in general terms) in Eira
#[derive(Debug, Clone)]
pub struct SignObject {
    pub name: String,
    pub values: HashMap<String, Value>,
    pub attunements: HashMap<String, SpellObject>,
}

impl SignObject {
    /// Creates a new SignObject with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: HashMap::new(),
            attunements: HashMap::new(),
        }
    }
}