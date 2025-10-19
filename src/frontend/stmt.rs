use crate::frontend::{expr::{Expr, WovenExpr}, reagents::{Reagent, WovenReagent}, scanner::Token, symbol_table::Symbol};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ExprStmt { expr: Expr },
    VarDeclaration { name: Token, mutable: bool, initializer: Option<Expr> },
    Fate { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While { condition: Expr, body: Box<Stmt> },
    Chant { expression: Expr },
    Block { statements: Vec<Stmt> },
    Sever,
    Spell { name: Token, reagents: Vec<Reagent>, body: Box<Stmt>, return_weave: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WovenStmt {
    ExprStmt { expr: WovenExpr },
    VarDeclaration { name: Token, mutable: bool, initializer: Option<WovenExpr>, symbol: Symbol },
    Fate { condition: WovenExpr, then_branch: Box<WovenStmt>, else_branch: Option<Box<WovenStmt>> },
    While { condition: WovenExpr, body: Box<WovenStmt> },
    Chant { expression: WovenExpr },
    Block { statements: Vec<WovenStmt> },
    Sever,
    Spell { name: Token, reagents: Vec<WovenReagent>, body: Box<WovenStmt>, symbol: Symbol }, // symbol contains ret weave's type
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
