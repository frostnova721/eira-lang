use crate::compiler::{Expr, WovenExpr, ast::decl::Decl, scanner::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ExprStmt {
        expr: Expr,
    },

    Fate {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Chant {
        expression: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
    Sever {
        token: Token,
    },
    Flow {
        token: Token,
    },

    Release {
        token: Token,
        expr: Option<Expr>,
    },

    Vanish {
        target: Expr,
        token: Token,
    },
    Declaration(Box<Decl>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WovenStmt {
    ExprStmt {
        expr: WovenExpr,
    },

    Fate {
        condition: WovenExpr,
        then_branch: Box<WovenStmt>,
        else_branch: Option<Box<WovenStmt>>,
    },
    While {
        condition: WovenExpr,
        body: Box<WovenStmt>,
    },
    Chant {
        expression: WovenExpr,
    },
    Block {
        statements: Vec<WovenStmt>,
    },
    Sever {
        token: Token,
    },
    Flow {
        token: Token,
    },

    Release {
        token: Token,
        expr: Option<WovenExpr>,
    },
    Declaration(Box<crate::compiler::ast::decl::WovenDecl>),
}
