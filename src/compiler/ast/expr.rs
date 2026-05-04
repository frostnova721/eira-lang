use crate::{
    compiler::{
        mark::{EtchedMark, WovenEtchedMark},
        scanner::Token,
        symbol_table::Symbol,
        weaves::Weave,
    },
    values::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: Token,
    },
    Unary {
        operand: Box<Expr>,
        operator: Token,
    },
    Literal {
        value: Value,
        token: Token,
    },
    Variable {
        name: Token,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Cast {
        reagents: Vec<Expr>,
        callee: Token,
    },
    Draw {
        marks: Vec<EtchedMark>,
        callee: Token,
    },
    Access {
        material: Box<Expr>,
        property: Token,
    },
    Deck {
        elements: Vec<Expr>,
        token: Token,
    },
    Extract {
        deck: Box<Expr>,
        index: Box<Expr>,
        token: Token,
    },
    DeckSet {
        deck: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
        token: Token,
    },
    FieldSet {
        material: Box<Expr>,
        property: Token,
        value: Box<Expr>,
    },
    Blank {
        token: Token,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WovenExpr {
    Binary {
        left: Box<WovenExpr>,
        right: Box<WovenExpr>,
        operator: Token,
        weave: Weave,
    },
    Unary {
        operand: Box<WovenExpr>,
        operator: Token,
        weave: Weave,
    },
    Literal {
        value: Value,
        token: Token,
        weave: Weave,
    },
    Variable {
        name: Token,
        weave: Weave,
        symbol: Symbol,
    },
    Grouping {
        expression: Box<WovenExpr>,
        weave: Weave,
    },
    Assignment {
        name: Token,
        value: Box<WovenExpr>,
        weave: Weave,
        symbol: Symbol,
    },
    Cast {
        reagents: Vec<WovenExpr>,
        callee: Token,
        weave: Weave,
        spell_symbol: Symbol,
    },
    Draw {
        marks: Vec<WovenEtchedMark>,
        callee: Token,
        weave: Weave,
        sign_symbol: Symbol,
    },
    Access {
        material: Box<WovenExpr>,
        property: Token,
        field_name_idx: u16,
        weave: Weave,
    },
    Deck {
        elements: Vec<WovenExpr>,
        weave: Weave,
    },
    Extract {
        deck: Box<WovenExpr>,
        index: Box<WovenExpr>,
        token: Token,
        weave: Weave,
    },
    DeckSet {
        deck: Box<WovenExpr>,
        index: Box<WovenExpr>,
        value: Box<WovenExpr>,
        token: Token,
        weave: Weave,
    },
    FieldSet {
        material: Box<WovenExpr>,
        property: Token,
        value: Box<WovenExpr>,
        field_name_idx: u16,
        weave: Weave,
    }
}

impl WovenExpr {
    pub fn weave(&self) -> Weave {
        match self {
            WovenExpr::Binary {
                left: _,
                right: _,
                operator: _,
                weave,
            } => weave.clone(),
            WovenExpr::Grouping {
                expression: _,
                weave,
            } => weave.clone(),
            WovenExpr::Literal {
                value: _,
                weave,
                token: _,
            } => weave.clone(),
            WovenExpr::Unary {
                operand: _,
                operator: _,
                weave,
            } => weave.clone(),
            WovenExpr::Variable {
                name: _,
                weave,
                symbol: _,
            } => weave.clone(),
            WovenExpr::Assignment {
                name: _,
                value: _,
                weave,
                symbol: _,
            } => weave.clone(),
            WovenExpr::Cast {
                reagents: _,
                callee: _,
                weave,
                spell_symbol: _,
            } => weave.clone(),
            WovenExpr::Draw {
                marks: _,
                callee: _,
                weave,
                sign_symbol: _,
            } => weave.clone(),
            WovenExpr::Access {
                material: _,
                property: _,
                field_name_idx: _,
                weave,
            } => weave.clone(),
            WovenExpr::Deck { elements: _, weave } => weave.clone(),
            WovenExpr::Extract {
                deck: _,
                index: _,
                token: _,
                weave,
            } => weave.clone(),
            WovenExpr::DeckSet {
                deck: _,
                index: _,
                value: _,
                token: _,
                weave,
            } => weave.clone(),
            WovenExpr::FieldSet {
                material: _,
                property: _,
                value: _,
                field_name_idx: _,
                weave,
            } => weave.clone(),
        }
    }

    // might stay unused
    pub fn symbol(&self) -> Option<&Symbol> {
        match self {
            WovenExpr::Variable {
                name: _,
                weave: _,
                symbol,
            } => Some(symbol),
            WovenExpr::Assignment {
                name: _,
                value: _,
                weave: _,
                symbol,
            } => Some(symbol),
            WovenExpr::Cast {
                reagents: _,
                callee: _,
                weave: _,
                spell_symbol,
            } => Some(spell_symbol),
            WovenExpr::Draw {
                marks: _,
                callee: _,
                weave: _,
                sign_symbol,
            } => Some(&sign_symbol),
            _ => None,
        }
    }

    pub fn token(&self) -> Token {
        match self {
            WovenExpr::Binary {
                left: _,
                right: _,
                operator,
                weave: _,
            } => operator.clone(),
            WovenExpr::Grouping {
                expression: _,
                weave: _,
            } => Token::dummy(),
            WovenExpr::Literal {
                value: _,
                weave: _,
                token,
            } => token.clone(),
            WovenExpr::Unary {
                operand: _,
                operator,
                weave: _,
            } => operator.clone(),
            WovenExpr::Variable {
                name,
                weave: _,
                symbol: _,
            } => name.clone(),
            WovenExpr::Assignment {
                name,
                value: _,
                weave: _,
                symbol: _,
            } => name.clone(),
            WovenExpr::Cast {
                reagents: _,
                callee,
                weave: _,
                spell_symbol: _,
            } => callee.clone(),
            WovenExpr::Draw {
                marks: _,
                callee,
                weave: _,
                sign_symbol: _,
            } => callee.clone(),
            WovenExpr::Access {
                material: _,
                property,
                field_name_idx: _,
                weave: _,
            } => property.clone(),
            WovenExpr::Deck {
                elements: _,
                weave: _,
            } => Token::dummy(),
            WovenExpr::Extract {
                deck: _,
                index: _,
                token,
                weave: _,
            } => token.clone(),
            WovenExpr::DeckSet {
                deck: _,
                index: _,
                value: _,
                token,
                weave: _,
            } => token.clone(),
            WovenExpr::FieldSet {
                material: _,
                property,
                value: _,
                field_name_idx: _,
                weave: _,
            } => property.clone(),
        }
    }
}
