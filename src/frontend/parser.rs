use std::rc::Rc;

use crate::{
    frontend::{
        expr::Expr,
        reagents::{Mark, Reagent},
        scanner::Token,
        stmt::Stmt,
        token_type::TokenType,
    },
    values::Value,
};

const MSG_MISSED_SEMICOLON: &str = "Expected a ';' after the expression. Forgot to add it?";
const MSG_BIND_VALUE_NOT_INITIALIZED: &str = "bind values must be initialized.";

pub struct Parser {
    // Token list
    tokens: Vec<Token>,

    // current token's index
    current_pos: usize,

    // Current and prev tokens
    previous: Token,
    current: Token,

    // error and unwinding
    panic: bool,
    error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let temp_token = Token {
            lexeme: "tempo tokan!".to_string(),
            line: 0,
            token_type: TokenType::Error, // temp
            column: 0,
        };

        let mut parser = Parser {
            tokens: tokens,
            current_pos: 0,
            previous: temp_token.clone(),
            current: temp_token,
            panic: false,
            error: false,
        };

        // parser.advance();
        parser.current = parser.tokens[0].clone();

        parser
    }

    pub fn parse(mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts: Vec<Stmt> = vec![];
        while !self.reached_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            }
        }

        if self.error {
            return Err(ParseError("Parsing failed due to errors.".to_owned()));
        }
        Ok(stmts)
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        if self.previous.token_type == TokenType::Eof {
            return;
        }
        loop {
            self.current_pos += 1;
            if self.current_pos >= self.tokens.len() {
                return;
            }
            self.current = self.tokens[self.current_pos].clone();
            // println!("pre: {:?}\ncur: {:?}", self.previous, self.current);
            if self.current.token_type != TokenType::Error {
                break;
            }

            // error token reporting!
            self.throw_error_at_current(&self.current.lexeme.clone());
        }
    }

    fn reached_end(&mut self) -> bool {
        self.current.token_type == TokenType::Eof
    }

    fn consume(&mut self, expect: TokenType, msg: &str) {
        if self.current.token_type == expect {
            self.advance();
            return;
        }

        self.throw_error_at_current(msg);
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&self, token_type: TokenType) -> bool {
        token_type == self.current.token_type
    }

    fn error_at(&mut self, msg: &str, pos: Token) {
        println!("msg {}", msg);
        if self.panic {
            return;
        }
        self.panic = true;
        println!(
            "Woah! Caught an incorrect magic at line: {}:{}\nError: {}",
            pos.line, pos.column, msg
        );
        self.error = true;
    }

    fn throw_error_at_current(&mut self, msg: &str) {
        self.error_at(msg, self.current.clone());
    }

    fn throw_error(&mut self, msg: &str) {
        self.error_at(msg, self.previous.clone());
    }

    fn parse_weave(&mut self, msg: &str) -> ParseResult<ParsedWeave> {
        self.consume(TokenType::Identifier, msg);
        let weave = self.previous.clone();
        let mut inner: Option<Token> = None;
        if self.match_token(TokenType::Less) {
            self.consume(TokenType::Identifier, "Expected a inner weave after '<'.");
            inner = Some(self.previous.clone());
            self.consume(
                TokenType::Greater,
                "Expected closing '>' after inner weave.",
            );
        }
        Ok(ParsedWeave {
            base: weave,
            inner: inner,
        })
    }

    fn sync(&mut self) {
        self.panic = false;

        while !self.reached_end() {
            if self.previous.token_type == TokenType::SemiColon {
                return;
            }

            match self.current.token_type {
                TokenType::Tome => return,
                TokenType::Spell => return,
                TokenType::Mark => return,
                TokenType::Bind => return,
                TokenType::Seal => return,
                TokenType::While => return,
                TokenType::Chant => return,
                TokenType::Release => return,
                TokenType::Fate => return,

                _ => {}
            }

            self.advance();
        }
    }

    // ----------------------- PARSE FUNCTIONS ----------------------//

    fn declaration(&mut self) -> Option<Stmt> {
        let res: ParseResult<Stmt>;
        if self.match_token(TokenType::Mark) {
            res = self.variable_declaration(true);
        } else if self.match_token(TokenType::Bind) {
            res = self.variable_declaration(false);
        } else if self.match_token(TokenType::Spell) {
            res = self.spell_declaration();
        } else {
            res = self.statement();
        }

        match res {
            Ok(res) => Some(res),
            Err(_) => {
                self.sync();
                None
            }
        }
    }

    fn spell_declaration(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::Identifier, "Expected a variable name!");
        let name = self.previous.clone();

        self.consume(TokenType::ParenLeft, "Expected '(' after spell name!");

        let mut params: Vec<Reagent> = vec![];

        if !self.check(TokenType::ParenRight) {
            loop {
                self.consume(TokenType::Identifier, "Expected reagent's name!");
                let token = self.previous.clone();

                // capture the weave of the mark!
                self.consume(TokenType::Colon, "Expected ':' for weave definition!");
                self.consume(
                    TokenType::Identifier,
                    "Expected a weave name to bind with the mark!",
                );

                let weave_name = self.previous.clone();
                params.push(Reagent {
                    name: token,
                    weave_name: weave_name,
                });

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::ParenRight, "Expected ')' after spell reagents!");

        let weave_name: Option<ParsedWeave>;

        if self.match_token(TokenType::ColonColon) {
            weave_name = Some(self.parse_weave("Expected a weave bound to the spell!")?);
            // weave_name = Some(self.previous.lexeme.clone());
        } else {
            weave_name = None;
        }

        self.consume(TokenType::BraceLeft, "Expected spell's working block!");
        let working = self.block()?;
        Ok(Stmt::Spell {
            name: name.clone(),
            reagents: params,
            body: Box::new(working),
            return_weave: weave_name,
        })
    }

    fn variable_declaration(&mut self, mutable: bool) -> ParseResult<Stmt> {
        // let thing = if mutable { "mark" } else { "bind" };
        self.consume(TokenType::Identifier, "Expected a variable name!");
        let name = self.previous.clone();
        let initializer: Option<Expr>;

        // TODO: Add explicit weave assigning

        if self.match_token(TokenType::Equal) {
            initializer = Some(self.expression()?);
        } else {
            if !mutable {
                self.throw_error(MSG_BIND_VALUE_NOT_INITIALIZED);
            }
            initializer = None;
        }
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::VarDeclaration {
            name: name,
            mutable: mutable,
            initializer: initializer,
        })
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token(TokenType::Chant) {
            self.chant_statment()
        } else if self.match_token(TokenType::BraceLeft) {
            self.block()
        } else if self.match_token(TokenType::Fate) {
            self.fate_statement()
        } else if self.match_token(TokenType::While) {
            self.while_statement()
        } else if self.match_token(TokenType::Sever) {
            self.sever_statement()
        } else if self.match_token(TokenType::Flow) {
            self.flow_statement()
        } else if self.match_token(TokenType::Release) {
            self.release_statement()
        } else if self.match_token(TokenType::Sign) {
            self.sign_statement()
        } else {
            self.expression_statement()
        }
    }

    fn block(&mut self) -> ParseResult<Stmt> {
        let mut stmts: Vec<Stmt> = vec![];
        while !self.check(TokenType::BraceRight) && !self.reached_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            } else {
                break;
            }
        }

        self.consume(
            TokenType::BraceRight,
            "Expected '}' at the end of a block. Forgot about it?",
        );

        Ok(Stmt::Block { statements: stmts })
    }

    fn release_statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token(TokenType::SemiColon) {
            return Ok(Stmt::Release {
                token: self.previous.clone(),
                expr: None,
            });
        }

        let expr = self.expression()?;
        let token = self.previous.clone();

        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);

        Ok(Stmt::Release {
            token,
            expr: Some(expr),
        })
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let e = self.expression()?;
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::ExprStmt { expr: e })
    }

    fn chant_statment(&mut self) -> ParseResult<Stmt> {
        let exp = self.expression()?;
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::Chant { expression: exp })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        let condition = self.expression()?;
        self.consume(TokenType::BraceLeft, "Expected '{' after loop condition.");
        let body = self.block()?;
        Ok(Stmt::While {
            condition: condition,
            body: Box::new(body),
        })
    }

    fn fate_statement(&mut self) -> ParseResult<Stmt> {
        let condition = self.expression()?;
        self.consume(TokenType::BraceLeft, "Expected '{' at start of fate block.");
        let then_branch = self.block()?;

        let else_branch = if self.match_token(TokenType::Divert) {
            self.consume(
                TokenType::BraceLeft,
                "Expected '{' at start of fate-else block.",
            );
            Some(Box::new(self.block()?))
        } else {
            None
        };
        Ok(Stmt::Fate {
            condition: condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch,
        })
    }

    fn sign_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::Identifier, "Expected a name for the sign.");
        let name = self.previous.clone();
        self.consume(TokenType::BraceLeft, "Expected '{' after the sign name.");

        let mut marks: Vec<Mark> = vec![];

        if !self.check(TokenType::BraceRight) {
            loop {
                self.consume(TokenType::Identifier, "Expected a mark name!");
                let mark_name = self.previous.clone();
                self.consume(
                    TokenType::Colon,
                    "Expected a weave definition for the field.",
                );
                self.consume(
                    TokenType::Identifier,
                    "Expected a weave name to bind with the sign's mark!",
                );
                let weave_name = self.previous.clone();

                marks.push(Mark {
                    name: mark_name,
                    weave_name,
                });

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::BraceRight, "Expected '}' after sign marks.");
        // self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);

        Ok(Stmt::Sign { name, marks })
    }

    fn sever_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::Sever {
            token: self.previous.clone(),
        })
    }

    fn flow_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::Flow {
            token: self.previous.clone(),
        })
    }

    // ----------------------- Expression stuff -------------------------------//
    fn expression(&mut self) -> ParseResult<Expr> {
        self.parse_precedence(Precedence::Assign)
    }

    fn grouping(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let exp = self.expression();
        self.consume(
            TokenType::ParenRight,
            "Close the bracket!\nError: Expected ')' after expression.",
        );
        Ok(Expr::Grouping {
            expression: Box::new(exp?),
        })
    }

    fn number(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let val: f64 = self.previous.lexeme.parse().unwrap();
        Ok(Expr::Literal {
            value: Value::Number(val),
            token: self.previous.clone(),
        })
    }

    fn literal(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        match self.previous.token_type {
            TokenType::True => Ok(Expr::Literal {
                value: Value::Bool(true),
                token: self.previous.clone(),
            }),
            TokenType::False => Ok(Expr::Literal {
                value: Value::Bool(false),
                token: self.previous.clone(),
            }),
            _ => Err(ParseError("Error: UNKNOWN.... LITERAL?!".to_owned())),
        }
    }

    fn string(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let string = self.previous.lexeme.clone();
        Ok(Expr::Literal {
            value: Value::String(Rc::new(string)),
            token: self.previous.clone(),
        })
    }

    fn unary(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let op = self.previous.clone();
        let exp = self.parse_precedence(Precedence::Unary)?;

        match op.token_type {
            TokenType::Minus | TokenType::Bang => Ok(Expr::Unary {
                operand: Box::new(exp),
                operator: op,
            }),
            _ => Err(ParseError("Unexpected! Verymuch!!".to_owned())),
        }
    }

    fn binary(&mut self, lhs: Expr) -> ParseResult<Expr> {
        let op = self.previous.clone();
        let rule = self.get_rule(op.token_type);
        let rhs = self.parse_precedence(rule.precedence.next())?;

        match op.token_type {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::BangEqual
            | TokenType::EqualEqual
            | TokenType::Less
            | TokenType::LessEqual
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Percent => Ok(Expr::Binary {
                left: Box::new(lhs),
                right: Box::new(rhs),
                operator: op,
            }),
            _ => Err(ParseError("idk anymore, maybe unreachable".to_owned())),
        }
    }

    fn cast(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        self.consume(TokenType::Identifier, "Expected a spell name to cast.");
        let spell_name = self.previous.clone();

        let mut reagents: Vec<Expr> = vec![];

        if self.match_token(TokenType::With) {
            // self.consume(TokenType::With, "Expected 'with' after spell name.");

            // Parse the reagent expressions
            loop {
                reagents.push(self.expression()?);
                if self.match_token(TokenType::Comma) {
                    continue;
                } else {
                    // End of reagent list when next token isn't a comma!
                    break;
                }
            }
        }

        Ok(Expr::Cast {
            reagents,
            callee: spell_name,
        })
    }

    fn variable(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let var_name = self.previous.clone();

        // if a '=' is found after the variable name, it should be a assignment
        if self.match_token(TokenType::Equal) {
            return Ok(Expr::Assignment {
                name: var_name,
                value: Box::new(self.expression()?),
            });
        }

        // else, it should be a variable access
        Ok(Expr::Variable { name: var_name })
    }

    // ----------------------- Core -------------------------------//

    fn parse_precedence(&mut self, precedence: Precedence) -> ParseResult<Expr> {
        self.advance();
        let rule = self.get_rule(self.previous.token_type).prefix;

        match rule {
            None => {
                self.throw_error("An expression was expected!");
                // return 0; // dummy stuff, might change later!
                return Err(ParseError("".to_owned()));
            }
            Some(prefix_rule) => {
                let can_assign = precedence.power() <= Precedence::Assign.power();
                let mut lhs = prefix_rule(self, can_assign);

                while precedence.power()
                    <= self.get_rule(self.current.token_type).precedence.power()
                {
                    self.advance();
                    // println!("{:?}", self.previous);
                    let infix_rule = self.get_rule(self.previous.token_type).infix.unwrap();
                    lhs = infix_rule(self, lhs?);
                }

                if can_assign && self.match_token(TokenType::Equal) {
                    self.throw_error("Assignment target provided is invalid! Take a look at it!");
                    // return 0;
                    return Err(ParseError("".to_owned()));
                }

                return lhs;
            }
        }
    }

    fn get_rule(&self, token_type: TokenType) -> ParseRule {
        match token_type {
            TokenType::Alias => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Self::unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Bind => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BraceLeft => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BraceRight => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Cast => ParseRule {
                prefix: Some(Self::cast),
                infix: None,
                precedence: Precedence::Call,
            },
            TokenType::Chant => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Colon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::ColonColon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Divert => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Dot => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::Call,
            },
            TokenType::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Equality,
            },
            TokenType::False => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Fate => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Flow => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(Self::variable),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::Mark => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Self::unary),
                infix: Some(Self::binary),
                precedence: Precedence::Term,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Self::number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::ParenLeft => ParseRule {
                prefix: Some(Self::grouping),
                infix: None,
                precedence: Precedence::Call,
            },
            TokenType::ParenRight => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Percent => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Term,
            },
            TokenType::Release => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Seal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::SemiColon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Sever => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Spell => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::String => ParseRule {
                prefix: Some(Self::string),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Tome => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::True => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            _ => panic!("Some rule went haywire!"),
        }
    }
}

struct ParseRule {
    prefix: Option<ParseFun>,
    infix: Option<InfixParseFun>,
    precedence: Precedence,
}

#[derive(Debug)] // Add this to allow printing the error
pub struct ParseError(pub String);

pub type ParseResult<T> = Result<T, ParseError>;

type ParseFun = fn(&mut Parser, bool) -> ParseResult<Expr>;

type InfixParseFun = fn(&mut Parser, Expr) -> ParseResult<Expr>;

#[derive(PartialEq, Debug, Clone)]
pub struct ParsedWeave {
    pub base: Token,
    pub inner: Option<Token>,
}

enum Precedence {
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

    // pub fn from_power(power: u8) -> Self {
    //     match power {
    //         0 => Precedence::None,
    //         1 => Precedence::Assign,
    //         2 => Precedence::Or,
    //         3 => Precedence::And,
    //         4 => Precedence::Equality,
    //         5 => Precedence::Compare,
    //         6 => Precedence::Term,
    //         7 => Precedence::Factor,
    //         8 => Precedence::Unary,
    //         9 => Precedence::Call,
    //         10 => Precedence::Primary,
    //         _ => panic!("Unknown precedence to match the power!"),
    //     }
    // }
}
