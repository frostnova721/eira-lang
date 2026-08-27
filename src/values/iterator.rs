use std::rc::Rc;

use crate::values::deck::DeckObject;

#[derive(Debug, Clone, PartialEq)]
pub enum IteratorState {
    Range { current: f64, end: f64 },
    Deck{ deck: Rc<DeckObject>, index: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IteratorObject {
    pub state: IteratorState,
}