use crate::frontend::{scanner::Token, weaves::Weave};

#[derive(Debug, Clone, PartialEq)]
pub struct Reagent {
    pub name: Token,
    pub weave_name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WovenReagent {
    pub name: Token,
    pub weave: Weave,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub name: Token,
    pub weave_name: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WovenMark {
    pub name: Token,
    pub weave: Weave,
}