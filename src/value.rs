use std::rc::Rc;

use crate::spell::{SpellObject};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Spell(Rc<SpellObject>),
    Emptiness,
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::String(_) => ValueType::String,
            Self::Bool(_) => ValueType::Bool,
            Self::Spell(_) => ValueType::Spell,
            Self::Emptiness => ValueType::Emptiness,
        }
    }
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_emptiness(&self) -> bool {
        matches!(self, Self::Emptiness)
    }

    pub fn is_falsey(&self) -> bool {
        matches!(self, Self::Bool(false))
    }

    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            _ => false,
        }
    }
}

pub fn print_value(value: Value) {
    match value {
        Value::Bool(value) => println!("{}", value),
        Value::Emptiness => println!("Emptiness"),
        Value::Number(value) => println!("{}", value),
        Value::String(value) => println!("{}", value),
        Value::Spell(spell) => println!("Spell '{}'", spell.name)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Emptiness
    }
}

enum ValueType {
    String,
    Number,
    Bool,
    Spell,
    Emptiness,
}