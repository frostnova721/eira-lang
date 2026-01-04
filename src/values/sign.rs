use std::collections::HashMap;

use crate::{SpellObject, Value};

/// Represents a Sign (or struct in general terms) in Eira
/// Marks -> fields/properties of the Sign, Since the magical signs consists of different marks
/// Attunements -> spells that are attuned to this sign
#[derive(Debug, Clone)]
pub struct SignObject {
    pub name: String,
    pub marks: HashMap<String, Value>,
    pub attunements: HashMap<String, SpellObject>,
}

impl SignObject {
    /// Creates a new SignObject with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            marks: HashMap::new(),
            attunements: HashMap::new(),
        }
    }
}