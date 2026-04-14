use crate::{Parser, Token, compiler::Expr};

pub struct ParseRule {
    pub prefix: Option<ParseFun>,
    pub infix: Option<InfixParseFun>,
    pub precedence: Precedence,
}

#[derive(Debug)] // Add this to allow printing the error
pub struct ParseError(pub String);

pub type ParseResult<T> = Result<T, ParseError>;

pub type ParseFun = fn(&mut Parser, bool) -> ParseResult<Expr>;

pub type InfixParseFun = fn(&mut Parser, Expr, bool) -> ParseResult<Expr>;

#[derive(PartialEq, Debug, Clone)]
pub struct ParsedWeave {
    pub base: Token,
    pub inner: Option<Box<ParsedWeave>>,
}

pub enum Precedence {
    None,
    Assign,
    Or,
    And,
    Equality,
    Compare,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn next(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assign,
            Precedence::Assign => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Compare,
            Precedence::Compare => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }

    pub fn power(&self) -> u8 {
        match self {
            Precedence::None => 0,
            Precedence::Assign => 1,
            Precedence::Or => 2,
            Precedence::And => 3,
            Precedence::Equality => 4,
            Precedence::Compare => 5,
            Precedence::Term => 6,
            Precedence::Factor => 7,
            Precedence::Unary => 8,
            Precedence::Call => 9,
            Precedence::Primary => 10,
        }
    }
}
