use crate::{frontend::{scanner::Token, tapestry::Tapestry}, runtime::value::Value};

#[derive(Debug, Clone)]
pub enum Expr {
    Binary { left: Box<Expr>, right: Box<Expr>, operator: Token },
    Unary { operand: Box<Expr>, operator: Token },
    Literal { value: Value },
    Variable { name: Token },
    Assignment { name: Token, value: Box<Expr> },
    Grouping { expression: Box<Expr> },
}

pub enum WovenExpr {
    Binary { left: Box<Expr>, right: Box<Expr>, operator: Token, tapestry: Tapestry },
    Unary { operand: Box<Expr>, operator: Token, tapestry: Tapestry },
    Literal { value: Value, tapestry: Tapestry },
    Variable { name: Token, tapestry: Tapestry },
    Assignment { name: Token, value: Box<Expr>, tapestry: Tapestry },
    Grouping { expression: Box<Expr>, tapestry: Tapestry },
}