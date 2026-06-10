use crate::{
    compiler::{
        Expr, WovenExpr,
        mark::{Mark, WovenMark},
        parser::types::ParsedWeave,
        reagents::{Reagent, WovenReagent},
        scanner::Token,
        symbol_table::Symbol,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ExprStmt {
        expr: Expr,
    },
    VarDeclaration {
        name: Token,
        mutable: bool,
        initializer: Option<Expr>,
        weave: Option<ParsedWeave>,
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
    Spell {
        name: Token,
        reagents: Vec<Reagent>,
        body: Box<Stmt>,
        return_weave: Option<ParsedWeave>,

        attuned_to: Option<Token>
    },
    Release {
        token: Token,
        expr: Option<Expr>,
    },
    Sign {
        name: Token,
        marks: Vec<Mark>,
    },
    Vanish {
        target: Expr,
        token: Token,
    },
    Attune {
        sign: Token,
        spells: Vec<Box<Stmt>>
    },
    Tether {
        token: Token,
        path: Vec<Token>,
        bind_to: Option<Token>,
        is_path: bool,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WovenStmt {
    ExprStmt {
        expr: WovenExpr,
    },
    VarDeclaration {
        name: Token,
        mutable: bool,
        initializer: Option<WovenExpr>,
        symbol: Symbol,
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
    Spell {
        name: Token,
        reagents: Vec<WovenReagent>,
        body: Box<WovenStmt>,
        spell_symbol: Symbol,
    },
    Release {
        token: Token,
        expr: Option<WovenExpr>,
    },
    Sign {
        name: Token,
        marks: Vec<WovenMark>,
        sign_symbol: Symbol,
    },
    Attune {
        sign: Token,
        spells: Vec<Box<WovenStmt>>,
    },
    Tether {
        token: Token,
        path: Vec<Token>,
        bind_to: Option<Token>,
        is_path: bool,
    },
}
