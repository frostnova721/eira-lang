use crate::{compiler::{mark::{EtchedMark, WovenEtchedMark}, scanner::Token, symbol_table::Symbol, tapestry::Tapestry}, values::{Value, sign::SignInfo}};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary { left: Box<Expr>, right: Box<Expr>, operator: Token },
    Unary { operand: Box<Expr>, operator: Token },
    Literal { value: Value, token: Token },
    Variable { name: Token },
    Grouping { expression: Box<Expr> },
    Assignment { name: Token, value: Box<Expr> },
    Cast { reagents: Vec<Expr>, callee: Token },
    Draw { marks: Vec<EtchedMark>, callee: Token },
    Access { material: Box<Expr>, property: Token },
    Deck { elements: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WovenExpr {
    Binary { left: Box<WovenExpr>, right: Box<WovenExpr>, operator: Token, tapestry: Tapestry },
    Unary { operand: Box<WovenExpr>, operator: Token, tapestry: Tapestry },
    Literal { value: Value, token: Token, tapestry: Tapestry },
    Variable { name: Token, tapestry: Tapestry, symbol: Symbol },
    Grouping { expression: Box<WovenExpr>, tapestry: Tapestry },
    Assignment { name: Token, value: Box<WovenExpr>, tapestry: Tapestry, symbol: Symbol },
    Cast { reagents: Vec<WovenExpr>, callee: Token, tapestry: Tapestry, spell_symbol: Symbol },
    Draw { marks: Vec<WovenEtchedMark>, callee: Token, tapestry: Tapestry, sign_info: SignInfo },
    Access { material: Box<WovenExpr>, property: Token, field_name_idx: u16, tapestry: Tapestry },
    Deck { elements: Vec<WovenExpr>, tapestry: Tapestry },
}

impl WovenExpr {
    pub fn tapestry(&self) -> Tapestry {
        match self {
            WovenExpr::Binary { left:_, right:_, operator:_, tapestry } => *tapestry,
            WovenExpr::Grouping { expression:_, tapestry } => *tapestry,
            WovenExpr::Literal { value:_, tapestry, token: _ } => *tapestry,
            WovenExpr::Unary { operand:_, operator:_, tapestry } => *tapestry,
            WovenExpr::Variable { name:_, tapestry, symbol:_ } => *tapestry,
            WovenExpr::Assignment { name:_, value:_, tapestry, symbol:_ } => *tapestry,
            WovenExpr::Cast { reagents:_, callee:_, tapestry, spell_symbol: _ } => *tapestry,
            WovenExpr::Draw { marks:_, callee:_, tapestry, sign_info: _ } => *tapestry,
            WovenExpr::Access { material:_, property:_, field_name_idx:_, tapestry } => *tapestry,
            WovenExpr::Deck { elements:_, tapestry } => *tapestry,
        }
    }

    // might stay unused 
    pub fn symbol(&self) -> Option<&Symbol> {
        match self {
            WovenExpr::Variable { name:_, tapestry:_, symbol } => Some(symbol),
            WovenExpr::Assignment { name:_, value:_, tapestry:_, symbol } => Some(symbol),
            WovenExpr::Cast { reagents:_, callee:_, tapestry:_, spell_symbol } => Some(spell_symbol),
            WovenExpr::Draw { marks:_, callee:_, tapestry:_, sign_info } => Some(&sign_info.symbol),
            _ => None
        }
    }

    pub fn token(&self) -> Token {
        match self {
            WovenExpr::Binary { left:_, right:_, operator, tapestry:_ } => operator.clone(),
            WovenExpr::Grouping { expression:_, tapestry:_ } => Token::dummy(),
            WovenExpr::Literal { value:_, tapestry:_, token } => token.clone(),
            WovenExpr::Unary { operand:_, operator, tapestry:_ } => operator.clone(),
            WovenExpr::Variable { name, tapestry:_, symbol:_ } => name.clone(),
            WovenExpr::Assignment { name, value:_, tapestry:_, symbol:_ } => name.clone(),
            WovenExpr::Cast { reagents:_, callee, tapestry:_, spell_symbol: _ } => callee.clone(),
            WovenExpr::Draw { marks:_, callee, tapestry:_, sign_info: _ } => callee.clone(),
            WovenExpr::Access { material:_, property, field_name_idx:_, tapestry:_ } => property.clone(),
            WovenExpr::Deck { elements:_, tapestry:_ } => Token::dummy(),
        }
    }
}