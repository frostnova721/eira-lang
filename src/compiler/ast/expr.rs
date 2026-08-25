use crate::{
    compiler::{
        mark::{EtchedMark, WovenEtchedMark},
        scanner::Token,
        symbol_table::Symbol,
        weaves::Weave,
    },
    values::{Value, native_spell::NativeSpell},
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
        callee: Box<Expr>,
        token: Token,
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
    Manifests {
        value: Box<Expr>,
        token: Token,
    },
    SafeAccess {
        material: Box<Expr>,
        property: Token,
    },
    AssertSafe {
        operand: Box<Expr>,
        operator: Token,
    },
    Cursed {
        span: (usize, usize),
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
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
    },
    Manifests {
        value: Box<WovenExpr>,
        token: Token,
        weave: Weave,
    },
    SafeAccess {
        material: Box<WovenExpr>,
        property: Token,
        field_name_idx: u16,
        weave: Weave,
    },
    AssertSafe {
        operand: Box<WovenExpr>,
        operator: Token,
        weave: Weave,
    },
    NativeCast {
        reagents: Vec<WovenExpr>,
        callee: Token,
        weave: Weave,
        native_spell: NativeSpell,
    },
    SafeCast {
        reagents: Vec<WovenExpr>,
        callee: Token,
        weave: Weave,
        spell_symbol: Symbol,
    },
    BoundSpell {
        is_safe: bool, // safe access or normal access
        material: Box<WovenExpr>,
        spell_symbol: Symbol,
        token: Token,
        weave: Weave,
    },
    Cursed {
        span: Option<(usize, usize)>,
    },
    Range {
        start: Box<WovenExpr>,
        end: Box<WovenExpr>,
        token: Token,
        weave: Weave,
    },
}

impl WovenExpr {
    pub fn weave(&self) -> Weave {
        match self {
            WovenExpr::Binary { weave, .. } => weave.clone(),
            WovenExpr::Grouping { weave, .. } => weave.clone(),
            WovenExpr::Literal { weave, .. } => weave.clone(),
            WovenExpr::Unary { weave, .. } => weave.clone(),
            WovenExpr::Variable { weave, .. } => weave.clone(),
            WovenExpr::Assignment { weave, .. } => weave.clone(),
            WovenExpr::Cast { weave, .. } => weave.clone(),
            WovenExpr::Draw { weave, .. } => weave.clone(),
            WovenExpr::Access { weave, .. } => weave.clone(),
            WovenExpr::Deck { elements: _, weave } => weave.clone(),
            WovenExpr::Extract { weave, .. } => weave.clone(),
            WovenExpr::DeckSet { weave, .. } => weave.clone(),
            WovenExpr::FieldSet { weave, .. } => weave.clone(),
            WovenExpr::Manifests { weave, .. } => weave.clone(),
            WovenExpr::SafeAccess { weave, .. } => weave.clone(),
            WovenExpr::AssertSafe { weave, .. } => weave.clone(),
            WovenExpr::NativeCast { weave, .. } => weave.clone(),
            WovenExpr::BoundSpell { weave, .. } => weave.clone(),
            WovenExpr::SafeCast { weave, .. } => weave.clone(),
            WovenExpr::Range {  weave, .. } => weave.clone(),
            WovenExpr::Cursed { .. } => Weave::Empty,
        }
    }

    // might stay unused
    pub fn symbol(&self) -> Option<&Symbol> {
        match self {
            WovenExpr::Variable { symbol, .. } => Some(symbol),
            WovenExpr::Assignment { symbol, .. } => Some(symbol),
            WovenExpr::Cast { spell_symbol, .. } => Some(spell_symbol),
            WovenExpr::Draw { sign_symbol, .. } => Some(&sign_symbol),
            WovenExpr::SafeCast { spell_symbol, .. } => Some(spell_symbol),
            _ => None,
        }
    }

    pub fn token(&self) -> Token {
        match self {
            WovenExpr::Binary { operator, .. } => operator.clone(),
            WovenExpr::Grouping { .. } => Token::dummy(),
            WovenExpr::Literal { token, .. } => token.clone(),
            WovenExpr::Unary { operator, .. } => operator.clone(),
            WovenExpr::Variable { name, .. } => name.clone(),
            WovenExpr::Assignment { name, .. } => name.clone(),
            WovenExpr::Cast { callee, .. } => callee.clone(),
            WovenExpr::Draw { callee, .. } => callee.clone(),
            WovenExpr::Access { property, .. } => property.clone(),
            WovenExpr::Deck { .. } => Token::dummy(),
            WovenExpr::Extract { token, .. } => token.clone(),
            WovenExpr::DeckSet { token, .. } => token.clone(),
            WovenExpr::FieldSet { property, .. } => property.clone(),
            WovenExpr::Manifests { token, .. } => token.clone(),
            WovenExpr::SafeAccess { property, .. } => property.clone(),
            WovenExpr::AssertSafe { operator, .. } => operator.clone(),
            WovenExpr::NativeCast { callee, .. } => callee.clone(),
            WovenExpr::BoundSpell { token, .. } => token.clone(),
            WovenExpr::SafeCast { callee, .. } => callee.clone(),
            WovenExpr::Cursed { .. } => Token::cursed(),
            WovenExpr::Range { token, .. } => token.clone(),
        }
    }
}
