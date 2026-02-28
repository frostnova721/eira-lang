use crate::{Token, frontend::{expr::Expr, weaves::Weave}};

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

#[derive(Debug, Clone, PartialEq)]
pub struct EtchedMark {
    pub name: Token,
    pub expr: Expr,
}