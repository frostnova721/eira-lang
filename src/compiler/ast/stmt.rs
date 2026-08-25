use crate::compiler::{Expr, WovenExpr, ast::decl::Decl, scanner::Token, symbol_table::Symbol};

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
    Cursed {
        span: Option<(usize, usize)>,
    },
    Cycle {
        iterable: Expr,  // a range or a deck or anything with ITERABLE strand
        variable: Token, // a variable
        body: Box<Stmt>, // should be a block statement
    },
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

    Cursed {
        span: Option<(usize, usize)>,
    },
    Cycle {
        iterator: WovenExpr,
        variable: Token,      // a variable
        body: Box<WovenStmt>, // should be a block statement
    },
}
