use crate::{frontend::{scanner::Token, tapestry::Tapestry}, runtime::value::Value};

#[derive(Debug, Clone)]
pub enum Expr {
    Binary { left: Box<Expr>, right: Box<Expr>, operator: Token },
    Unary { operand: Box<Expr>, operator: Token },
    Literal { value: Value },
    Variable { name: Token },
    Grouping { expression: Box<Expr> },
}

#[derive(Debug, Clone)]
pub enum WovenExpr {
    Binary { left: Box<WovenExpr>, right: Box<WovenExpr>, operator: Token, tapestry: Tapestry },
    Unary { operand: Box<WovenExpr>, operator: Token, tapestry: Tapestry },
    Literal { value: Value, tapestry: Tapestry },
    Variable { name: Token, tapestry: Tapestry },
    Grouping { expression: Box<WovenExpr>, tapestry: Tapestry },
}

impl WovenExpr {
    pub fn tapestry(&self) -> Tapestry {
        match self {
            WovenExpr::Binary { left:_, right:_, operator:_, tapestry } => *tapestry,
            WovenExpr::Grouping { expression:_, tapestry } => *tapestry,
            WovenExpr::Literal { value:_, tapestry } => *tapestry,
            WovenExpr::Unary { operand:_, operator:_, tapestry } => *tapestry,
            WovenExpr::Variable { name:_, tapestry } => *tapestry,
        }
    } 
}