use std::{collections::HashMap, rc::Rc};

use crate::{
    SpellObject, Value,
    frontend::{reagents::Mark, weaves::Weave},
    values::spell::SpellInfo,
};

/// Represents a Sign (or struct in general terms) in Eira
/// Marks -> fields/properties of the Sign, Since the magical signs consists of different marks
/// Attunements -> spells that are attuned to this sign
#[derive(Debug, Clone)]
pub struct SignObject {
    pub schema: Rc<SignSchema>,
    pub marks: Vec<Value>,
    pub attunements: HashMap<String, SpellObject>,
}

impl SignObject {
    /// Creates a new SignObject with the given schema
    pub fn new(schema: Rc<SignSchema>) -> Self {
        let field_count = schema.field_count();
        Self {
            schema: schema,
            marks: vec![Value::Emptiness; field_count],
            attunements: HashMap::new(),
        }
    }

    pub fn set_field(&mut self, index: usize, value: Value) -> Result<(), String> {
        if index > self.marks.len() {
            return Err(format!("Field index {} out of bounds.", index));
        }

        self.marks[index] = value;
        // self.schema.field_names;
        Ok(())
    }

    pub fn get_field(&self, index: usize) -> Value {
        self.marks[index].clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignSchema {
    pub name: String,
    pub field_indices: HashMap<String, usize>,
    pub field_names: Vec<String>,
    pub field_weaves: Vec<Weave>,
}

impl SignSchema {
    pub fn new(name: String) -> SignSchema {
        SignSchema {
            name,
            field_indices: HashMap::new(),
            field_names: vec![],
            field_weaves: vec![],
        }
    }

    pub fn add_field(&mut self, name: String, weave: Weave) {
        let idx = self.field_names.len();
        self.field_indices.insert(name.clone(), idx);
        self.field_names.push(name);
        self.field_weaves.push(weave);
    }

    pub fn get_field_index(&self, name: String) -> Option<usize> {
        self.field_indices.get(&name).copied()
    }

    pub fn field_count(&self) -> usize {
        self.field_names.len()
    }
}

pub struct SignInfo {
    pub name: String,
    pub marks: HashMap<String, Weave>,
    pub attunements: HashMap<String, SpellInfo>,
}
