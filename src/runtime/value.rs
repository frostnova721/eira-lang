use std::rc::Rc;

use crate::runtime::spell::{ClosureObject, SpellObject};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Closure(Rc<ClosureObject>),
    Emptiness,
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::String(_) => ValueType::String,
            Self::Bool(_) => ValueType::Bool,
            Self::Closure(_) => ValueType::Closure,
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

    pub fn is_closure(&self) -> bool {
        matches!(self, Self::Closure(_))
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

impl From<bool> for Value {
    fn from(val: bool) -> Value {
        Value::Bool(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Value {
        Value::Number(val)
    }
}

impl From<String> for Value {
    fn from(val: String) -> Value {
        Value::String(Rc::new(val))
    }
}

pub fn print_value(value: Value) {
    match value {
        Value::Bool(value) => println!("{}", value),
        Value::Emptiness => println!("Emptiness"),
        Value::Number(value) => println!("{}", value),
        Value::String(value) => println!("{}", value),
        Value::Closure(closure) => println!("Closure '{:?}'", closure.spell.name),
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
    Closure,
    Emptiness,
}
