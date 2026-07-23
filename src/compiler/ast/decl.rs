use crate::{
    Token,
    compiler::{
        Expr, Stmt, WovenExpr, WovenStmt,
        mark::{Mark, WovenMark},
        parser::types::ParsedWeave,
        reagents::{Reagent, WovenReagent},
        symbol_table::Symbol,
        types::Visibility,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    VarDeclaration {
        name: Token,
        mutable: bool,
        initializer: Option<Expr>,
        weave: Option<ParsedWeave>,
        visibility: Option<Visibility>,
    },

    Spell {
        name: Token,
        reagents: Vec<Reagent>,
        body: Box<Stmt>,
        return_weave: Option<ParsedWeave>,
        visibility: Option<Visibility>,
        attuned_to: Option<Token>,
    },

    Sign {
        name: Token,
        marks: Vec<Mark>,
        visibility: Option<Visibility>,
    },

    Attune {
        sign: Token,
        spells: Vec<Box<Stmt>>,
    },
    Tether {
        token: Token,
        path: Vec<Token>,
        bind_to: Option<Token>,
        is_path: bool,
    },
    Statement { stmt: Box<Stmt>, token: Token },

    Cursed {
        span: (usize, usize),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WovenDecl {
    VarDeclaration {
        name: Token,
        mutable: bool,
        initializer: Option<WovenExpr>,
        symbol: Symbol,
    },

    Spell {
        name: Token,
        reagents: Vec<WovenReagent>,
        body: Box<WovenStmt>,
        spell_symbol: Symbol,
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
        statements: Vec<WovenDecl>,
        path: String,
        bind_to: Option<Token>,
    },
    Statement { stmt: Box<WovenStmt>, token: Token },

     Cursed {
        span: (usize, usize),
    }
}
