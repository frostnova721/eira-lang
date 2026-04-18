use crate::compiler::{parser::types::ParsedWeave, scanner::Token, weaves::Weave};

#[derive(Debug, Clone, PartialEq)]
pub struct Reagent {
    pub name: Token,
    pub weave: ParsedWeave,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WovenReagent {
    pub name: Token,
    pub weave: Weave,
}
