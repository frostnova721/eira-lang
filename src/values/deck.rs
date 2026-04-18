use std::cell::RefCell;

use crate::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckObject {
    pub items: RefCell<Vec<Value>>,
    pub capacity: Option<usize>,
}

impl DeckObject {
    pub fn new(items: Vec<Value>, capacity: Option<usize>) -> DeckObject {
        DeckObject {
            items: RefCell::new(items), capacity,
        }
    }
}
