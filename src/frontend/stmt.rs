use crate::frontend::{expr::Expr, scanner::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt { expr: Expr },
    VarDeclaration { name: Token, mutable: bool, initializer: Option<Expr> },
    Fate { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While { condition: Expr, body: Box<Stmt> },
    Chant { expression: Expr },
    Block { statements: Vec<Stmt> },
    Sever,
}