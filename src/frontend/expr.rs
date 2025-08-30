use crate::{frontend::scanner::Token, runtime::value::Value};

#[derive(Debug)]
pub enum Expr {
    Binary { left: Box<Expr>, right: Box<Expr>, operator: Token, weave: Option<u64> },
    Unary { operand: Box<Expr>, operator: Token, weave: Option<u64> },
    Literal { value: Value, weave: Option<u64> },
    Variable { name: Token, weave: Option<u64> },
    Assignment { name: Token, value: Box<Expr>, weave: Option<u64> },
    Grouping { expression: Box<Expr>, weave: Option<u64> },
}