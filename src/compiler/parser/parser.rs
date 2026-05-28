use crate::compiler::{
    Expr, Stmt,
    parser::types::{ParseError, ParseResult, ParseRule, ParsedWeave, Precedence},
    scanner::Token,
    token_type::TokenType,
};

pub(super) const MSG_BIND_VALUE_NOT_INITIALIZED: &str = "bind values must be initialized.";
pub(super) const MSG_MISSED_SEMICOLON: &str =
    "Expected a ';' after the expression. Forgot to add it?";

pub struct Parser {
    // Token list
    pub(super) tokens: Vec<Token>,
    pub(super) current_file: String,

    // current token's index
    pub(super) current_pos: usize,

    // Current and prev tokens
    pub(super) previous: Token,
    pub(super) current: Token,

    // error and unwinding
    pub(super) panic: bool,
    pub(super) error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, current_file: String) -> Self {
        let temp_token = Token {
            lexeme: "tempo tokan!".to_string(),
            line: 0,
            token_type: TokenType::Error, // temp
            column: 0,
        };

        let mut parser = Parser {
            tokens: tokens,
            current_file,
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

    pub(super) fn advance(&mut self) {
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

    pub(super) fn reached_end(&mut self) -> bool {
        self.current.token_type == TokenType::Eof
    }

    pub(super) fn consume(&mut self, expect: TokenType, msg: &str) {
        if self.current.token_type == expect {
            self.advance();
            return;
        }

        self.throw_error_at_current(msg);
    }

    pub(super) fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        true
    }

    pub(super) fn check(&self, token_type: TokenType) -> bool {
        token_type == self.current.token_type
    }

    pub(super) fn error_at(&mut self, msg: &str, pos: Token) {
        if self.panic {
            // return;
        } else {
            self.panic = true;
        }
        println!(
            "Woah! Caught an incorrect magic at: {}:{}:{}\nError: {}\n",
            self.current_file, pos.line, pos.column, msg
        );
        self.error = true;
    }

    pub(super) fn throw_error_at_current(&mut self, msg: &str) {
        self.error_at(msg, self.current.clone());
    }

    pub(super) fn throw_error(&mut self, msg: &str) {
        self.error_at(msg, self.previous.clone());
    }

    pub(super) fn parse_weave(&mut self, err_msg: &str) -> ParseResult<ParsedWeave> {
        self.consume(TokenType::Identifier, err_msg);
        let weave = self.previous.clone();
        let mut inner: Option<Box<ParsedWeave>> = None;
        let mut capacity: Option<usize> = None;

        if self.match_token(TokenType::Less) {
            inner = Some(Box::new(self.parse_weave(
                "Expected a weave name to bind with the weave after the '<'!",
            )?));

            if self.match_token(TokenType::Comma) {
                self.consume(
                    TokenType::Number,
                    "Expected a capacity for the weave after ','!",
                );
                capacity = Some(self.previous.lexeme.parse::<usize>().unwrap());
            }

            self.consume(
                TokenType::Greater,
                "Expected closing '>' after inner weave.",
            );
        }
        Ok(ParsedWeave {
            base: weave,
            inner: inner,
            capacity: capacity,
        })
    }

    pub(super) fn sync(&mut self) {
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
                TokenType::Sign => return,
                _ => {}
            }

            self.advance();
        }
    }

    // ----------------------- PARSE FUNCTIONS ----------------------//

    pub(super) fn declaration(&mut self) -> Option<Stmt> {
        let res: ParseResult<Stmt>;
        if self.match_token(TokenType::Mark) {
            res = self.variable_declaration(true);
        } else if self.match_token(TokenType::Bind) {
            res = self.variable_declaration(false);
        } else if self.match_token(TokenType::Spell) {
            res = self.spell_declaration(None);
        } else if self.match_token(TokenType::Sign) {
            res = self.sign_declaration();
        } else if self.match_token(TokenType::Attune) {
            res = self.attune_declaration();
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

    // --------------------- Statements ---------------------- //

    pub(super) fn statement(&mut self) -> ParseResult<Stmt> {
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
        } else if self.match_token(TokenType::Vanish) {
            self.vanish_statement()
        } else {
            self.expression_statement()
        }
    }

    // ----------------------- Expression stuff -------------------------------//
    pub(super) fn expression(&mut self) -> ParseResult<Expr> {
        self.parse_precedence(Precedence::Assign)
    }

    // ----------------------- Core -------------------------------//

    pub(super) fn parse_precedence(&mut self, precedence: Precedence) -> ParseResult<Expr> {
        self.advance();
        let rule = self.get_rule(self.previous.token_type).prefix;

        match rule {
            None => {
                self.throw_error("An expression was expected!");
                return Err(ParseError("".to_owned()));
            }
            Some(prefix_rule) => {
                let can_assign = precedence.power() <= Precedence::Assign.power();
                let mut lhs = prefix_rule(self, can_assign)?;

                while precedence.power()
                    <= self.get_rule(self.current.token_type).precedence.power()
                {
                    let prev_prev = self.previous.clone();
                    self.advance();
                    let Some(infix_rule) = self.get_rule(self.previous.token_type).infix else {
                        self.error_at(
                            &format!(
                                "'{}' is not an infix operator. But was used as one!",
                                self.previous.lexeme
                            ),
                            prev_prev,
                        );
                        break;
                    };

                    lhs = infix_rule(self, lhs, can_assign)?;
                }

                if can_assign && self.match_token(TokenType::Equal) {
                    let equals = self.previous.clone();
                    let value = self.expression()?;

                    match lhs {
                        Expr::Variable { name } => {
                            return Ok(Expr::Assignment {
                                name,
                                value: Box::new(value),
                            });
                        }
                        Expr::Extract {
                            deck,
                            index,
                            token: _,
                        } => {
                            return Ok(Expr::DeckSet {
                                deck,
                                index,
                                value: Box::new(value),
                                token: equals,
                            });
                        }
                        Expr::Access { material, property } => {
                            return Ok(Expr::FieldSet {
                                material,
                                property,
                                value: Box::new(value),
                            });
                        }
                        _ => {}
                    }

                    self.throw_error("Assignment target provided is invalid! Take a look at it!");
                    // return 0;
                    return Err(ParseError("".to_owned()));
                }

                return Ok(lhs);
            }
        }
    }

    pub(super) fn get_rule(&self, token_type: TokenType) -> ParseRule {
        match token_type {
            TokenType::Bang => ParseRule {
                prefix: Some(Self::unary),
                infix: Some(Self::assert_safe),
                precedence: Precedence::Call,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Equality,
            },
            TokenType::BraceLeft => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Cast => ParseRule {
                prefix: Some(Self::cast),
                infix: None,
                precedence: Precedence::Call,
            },
            TokenType::Dot => ParseRule {
                prefix: None,
                infix: Some(Self::access),
                precedence: Precedence::Call,
            },
            TokenType::Ego => ParseRule {
                prefix: Some(Self::variable),
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
            TokenType::InterpolateStart => ParseRule {
                prefix: Some(Self::string),
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
            TokenType::Manifests => ParseRule {
                prefix: None,
                infix: Some(Self::manifests),
                precedence: Precedence::Call,
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
            TokenType::QuestionDot => ParseRule {
                prefix: None,
                infix: Some(Self::safe_access),
                precedence: Precedence::Call,
            },
            TokenType::Seal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::SquareLeft => ParseRule {
                prefix: Some(Self::deck),
                infix: Some(Self::extract),
                precedence: Precedence::Call,
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
            TokenType::Tilde => ParseRule {
                prefix: Some(Self::draw),
                infix: None,
                precedence: Precedence::Call,
            },
            TokenType::True => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Underscore => ParseRule {
                prefix: Some(Self::blank),
                infix: None,
                precedence: Precedence::None,
            },
            // default rule
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}
