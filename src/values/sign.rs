use std::collections::HashMap;

use crate::{SpellObject, Value, frontend::weaves::Weave, values::spell::SpellInfo};

/// Represents a Sign (or struct in general terms) in Eira
/// Marks -> fields/properties of the Sign, Since the magical signs consists of different marks
/// Attunements -> spells that are attuned to this sign
#[derive(Debug, Clone)]
pub struct SignObject {
    pub name: String,
    pub marks: HashMap<String, Value>,
    pub attunements: HashMap<String, SpellObject>,
}

pub struct SignInfo {
    pub name: String,
    pub marks: HashMap<String, Weave>,
    pub attunements: HashMap<String, SpellInfo>,
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