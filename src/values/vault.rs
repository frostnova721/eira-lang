use crate::Value;
use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

/// Immutable, hash-stable keys. Numeric zero is canonicalized; NaN is rejected.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VaultKey {
    Number(u64),
    Text(Rc<String>),
    Truth(bool),
}
impl VaultKey {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Number(n) if n.is_nan() => Err("NaN cannot be a Vault key.".into()),
            Value::Number(n) => Ok(Self::Number(if *n == 0.0 { 0 } else { n.to_bits() })),
            Value::String(s) => Ok(Self::Text(Rc::clone(s))),
            Value::Bool(b) => Ok(Self::Truth(*b)),
            _ => Err("Vault keys must be Num, Text, or Truth.".into()),
        }
    }
    pub fn to_value(&self) -> Value {
        match self {
            Self::Number(n) => Value::Number(f64::from_bits(*n)),
            Self::Text(s) => Value::String(Rc::clone(s)),
            Self::Truth(b) => Value::Bool(*b),
        }
    }
}

#[derive(Debug)]
pub struct VaultObject {
    entries: RefCell<HashMap<VaultKey, Value>>,
    pub max_entries: Option<usize>,
}
impl VaultObject {
    pub fn new(max_entries: Option<usize>, hint: usize) -> Result<Self, String> {
        let mut entries = HashMap::new();
        entries
            .try_reserve(hint.min(max_entries.unwrap_or(hint)))
            .map_err(|_| "Cannot allocate Vault storage.".to_string())?;
        Ok(Self {
            entries: RefCell::new(entries),
            max_entries,
        })
    }
    pub fn insert(&self, key: &Value, value: Value) -> Result<(), String> {
        let key = VaultKey::from_value(key)?;
        let mut entries = self.entries.borrow_mut();
        let full = self.max_entries.is_some_and(|n| entries.len() >= n);
        // One table lookup; replacing an existing key is legal even when full.
        match entries.entry(key) {
            Entry::Occupied(mut slot) => {
                slot.insert(value);
            }
            Entry::Vacant(slot) => {
                if full {
                    return Err("Vault maximum entry count exceeded.".into());
                }
                slot.insert(value);
            }
        }
        Ok(())
    }
    pub fn get(&self, key: &Value) -> Result<Value, String> {
        Ok(self
            .entries
            .borrow()
            .get(&VaultKey::from_value(key)?)
            .cloned()
            .unwrap_or(Value::Emptiness))
    }
    pub fn contains(&self, key: &Value) -> Result<bool, String> {
        Ok(self
            .entries
            .borrow()
            .contains_key(&VaultKey::from_value(key)?))
    }
    pub fn remove(&self, key: &Value) -> Result<Value, String> {
        Ok(self
            .entries
            .borrow_mut()
            .remove(&VaultKey::from_value(key)?)
            .unwrap_or(Value::Emptiness))
    }
    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }
    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
    }
    pub fn keys(&self) -> Vec<Value> {
        self.entries
            .borrow()
            .keys()
            .map(VaultKey::to_value)
            .collect()
    }
    pub fn values(&self) -> Vec<Value> {
        self.entries.borrow().values().cloned().collect()
    }
}
