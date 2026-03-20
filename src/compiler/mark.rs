use crate::{Token, compiler::{expr::{Expr, WovenExpr}, weaves::Weave}};

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

#[derive(Debug, Clone, PartialEq)]
pub struct WovenEtchedMark {
    pub name: Token,
    pub expr: WovenExpr,
}