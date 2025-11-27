use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::values::spell::{ClosureObject, SpellObject};

/// The value's container for runtime
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Closure(Rc<ClosureObject>),
    Spell(Rc<SpellObject>),
    Sign(),
    Emptiness,
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::String(_) => ValueType::String,
            Self::Bool(_) => ValueType::Bool,
            Self::Closure(_) => ValueType::Closure,
            Self::Spell(_) => ValueType::Spell,
            Self::Emptiness => ValueType::Emptiness,
            Self::Sign() => ValueType::Struct,
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

    pub fn is_spell(&self) -> bool {
        matches!(self, Self::Spell(_))
    }

    pub fn extract_number(&self) -> Option<f64> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn extract_string(&self) -> Option<String> {
        if let Value::String(s) = self {
            Some(s.to_string())
        } else {
            None
        }
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare numbers by their bits to handle all cases consistently
            (Self::Number(a), Self::Number(b)) => a.to_bits() == b.to_bits(),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Emptiness, Self::Emptiness) => true,
            // Closures are unique runtime objects and should not be considered equal
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            // For numbers, hash their raw bit representation
            Self::Number(n) => n.to_bits().hash(state),
            Self::String(s) => s.hash(state),
            Self::Bool(b) => b.hash(state),
            Self::Emptiness => {}  //hmm
            Self::Closure(_) => {} // not a compile time const
            Self::Spell(_) => {}   // not a compile time const
            Self::Sign() => {}     // not a compile time const
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

// responsible for converting the value to a identifiable string (like toString())
pub fn print_value(value: Value) {
    match value {
        Value::Bool(value) => println!("{}", value),
        Value::Emptiness => println!("Emptiness"),
        Value::Number(value) => println!("{}", value),
        Value::String(value) => println!("{}", value),
        Value::Closure(closure) => println!("Spell '{}'", closure.spell.name.clone().unwrap()),
        Value::Spell(spell) => println!("Spell '{}'", spell.name.clone().unwrap()),
        Value::Sign() => println!("Sign"),
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Emptiness
    }
}

pub enum ValueType {
    String,
    Number,
    Bool,
    Closure,
    Spell,
    Struct,
    Emptiness,
}
