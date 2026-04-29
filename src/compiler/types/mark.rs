use crate::{
    Token,
    compiler::{Expr, WovenExpr, parser::types::ParsedWeave, weaves::Weave},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub name: Token,
    pub parsed_weave: ParsedWeave,
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
