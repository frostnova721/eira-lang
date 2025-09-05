use crate::frontend::{expr::{Expr, WovenExpr}, scanner::Token, strands::NO_STRAND, tapestry::{self, Tapestry}};

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

pub enum WovenStmt {
    ExprStmt { expr: WovenExpr },
    VarDeclaration { name: Token, mutable: bool, initializer: Option<WovenExpr> },
    Fate { condition: WovenExpr, then_branch: Box<WovenStmt>, else_branch: Option<Box<WovenStmt>> },
    While { condition: WovenExpr, body: Box<WovenStmt> },
    Chant { expression: WovenExpr },
    Block { statements: Vec<WovenStmt> },
    Sever,
}

// impl WovenStmt {
//     pub fn tapestry(&self) -> Tapestry {
//         match self {
//             WovenStmt::ExprStmt { expr:_, tapestry } => *tapestry,
//             WovenStmt::VarDeclaration { name:_, mutable:_, initializer:_, tapestry } => *tapestry,
//             WovenStmt::Fate { condition:_, then_branch:_, else_branch:_, tapestry } => *tapestry,
//             WovenStmt::While { condition:_, body:_ } => Tapestry::new(NO_STRAND),
//             WovenStmt::Chant { expression:_, tapestry } => *tapestry,
//             WovenStmt::Block { statements:_, tapestry } => *tapestry,
//             WovenStmt::Sever => Tapestry::new(NO_STRAND),
//         }
//     }
// }
