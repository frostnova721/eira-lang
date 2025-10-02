use crate::frontend::{scanner::Token, weaves::Weave};

#[derive(Debug, Clone, PartialEq)]
pub struct Reagent {
    pub name: Token,
    pub weave_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WovenReagent {
    pub name: Token,
    pub weave: Weave,
}