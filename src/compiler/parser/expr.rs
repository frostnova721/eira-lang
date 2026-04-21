use std::rc::Rc;

use crate::{Parser, Token, Value, compiler::{Expr, mark::EtchedMark, parser::types::{ParseError, ParseResult, Precedence}, token_type::TokenType}};

impl Parser {
      pub(super) fn grouping(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let exp = self.expression();
        self.consume(
            TokenType::ParenRight,
            "Close the bracket!\nError: Expected ')' after expression.",
        );
        Ok(Expr::Grouping {
            expression: Box::new(exp?),
        })
    }

    pub(super) fn number(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let val: f64 = self.previous.lexeme.parse().unwrap();
        Ok(Expr::Literal {
            value: Value::Number(val),
            token: self.previous.clone(),
        })
    }

    pub(super) fn literal(&mut self, _can_assign: bool) -> ParseResult<Expr> {
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

    pub(super) fn string(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let string = if self.previous.token_type == TokenType::String {
            self.previous.lexeme.clone()
        } else {
            "".to_owned()
        };

        let mut expr = Expr::Literal {
            value: Value::String(Rc::new(string)),
            token: self.previous.clone(),
        };

        // Handle case where string starts with interpolation
        if self.previous.token_type == TokenType::InterpolateStart {
            let inner_expr = self.expression()?;
            let plus = Token {
                column: self.previous.column,
                line: self.previous.line,
                token_type: TokenType::Plus,
                lexeme: self.previous.lexeme.clone(),
            };

            self.consume(
                TokenType::InterpolateEnd,
                "Expected ')' after interpolated expression.",
            );

            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(inner_expr),
                operator: plus,
            };
        }

        loop {
            while self.match_token(TokenType::InterpolateStart) {
                let inner_expr = self.expression()?;
                let plus = Token {
                    column: self.previous.column,
                    line: self.previous.line,
                    token_type: TokenType::Plus,
                    lexeme: self.previous.lexeme.clone(),
                };

                self.consume(
                    TokenType::InterpolateEnd,
                    "Expected ')' after interpolated expression.",
                );

                expr = Expr::Binary {
                    left: Box::new(expr),
                    right: Box::new(inner_expr),
                    operator: plus,
                };
            }

            if self.match_token(TokenType::String) {
                let next_str = Expr::Literal {
                    value: Value::String(Rc::new(self.previous.lexeme.clone())),
                    token: self.previous.clone(),
                };

                expr = Expr::Binary {
                    left: Box::new(expr),
                    right: Box::new(next_str),
                    operator: Token {
                        column: self.previous.column,
                        line: self.previous.line,
                        token_type: TokenType::Plus,
                        lexeme: self.previous.lexeme.clone(),
                    },
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    pub(super) fn unary(&mut self, _can_assign: bool) -> ParseResult<Expr> {
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

    pub(super) fn binary(&mut self, lhs: Expr, _can_assign: bool) -> ParseResult<Expr> {
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
            _ => Err(ParseError("idk anymore, unreachable ig".to_owned())),
        }
    }

    pub(super) fn cast(&mut self, _can_assign: bool) -> ParseResult<Expr> {
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

    pub(super) fn draw(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        self.consume(TokenType::Identifier, "Expected a Sign name to draw!");
        let sign_name = self.previous.clone();

        let mut marks: Vec<EtchedMark> = vec![];

        if self.match_token(TokenType::With) {
            self.consume(TokenType::BraceLeft, "Expected '{' after 'with'");

            // Parse the marks expressions
            loop {
                if self.match_token(TokenType::Identifier) {
                    // self.consume(TokenType::Identifier, "Expected a mark name!");
                    let param = self.previous.clone();

                    self.consume(TokenType::Colon, "Expected a ':' after the mark name.");

                    let mrk = EtchedMark {
                        expr: self.expression()?,
                        name: param,
                    };

                    marks.push(mrk);

                    if self.match_token(TokenType::Comma) {
                        continue;
                    } else {
                        // End of mark list when next token isn't a comma!
                        break;
                    }
                } else {
                    // simply break out of the loop, any errors will be caught by following code (hopefully)
                    break;
                }
            }

            self.consume(
                TokenType::BraceRight,
                "Expected '}' after defining the marks for the sign!",
            );
        }

        Ok(Expr::Draw {
            marks,
            callee: sign_name,
        })
    }

    pub(super) fn access(&mut self, lhs: Expr, _can_assign: bool) -> ParseResult<Expr> {
        // nothing much to do here than just getting the property name.
        self.consume(TokenType::Identifier, "Expected a property name after '.'!");
        Ok(Expr::Access {
            material: Box::new(lhs),
            property: self.previous.clone(),
        })
    }

    pub(super) fn variable(&mut self, _can_assign: bool) -> ParseResult<Expr> {
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

    pub(super) fn deck(&mut self, _can_assign: bool) -> ParseResult<Expr> {
        let mut elements: Vec<Expr> = vec![];

        // Handle empty deck case []
        if self.check(TokenType::SquareRight) {
            self.advance();
            return Ok(Expr::Deck {
                elements,
                token: self.previous.clone(),
            });
        }

        // Parse elements
        loop {
            elements.push(self.expression()?);

            if self.match_token(TokenType::Comma) && !self.check(TokenType::SquareRight) {
                continue;
            } else {
                break;
            }
        }

        self.consume(TokenType::SquareRight, "Expected ']' after deck elements.");
        Ok(Expr::Deck {
            elements,
            token: self.previous.clone(),
        })
    }

    pub(super) fn extract(&mut self, lhs: Expr, _can_assign: bool) -> ParseResult<Expr> {
        let index_expr = self.expression()?;
        self.consume(
            TokenType::SquareRight,
            "Expected ']' after deck access expression.",
        );

        Ok(Expr::Extract {
            deck: Box::new(lhs),
            index: Box::new(index_expr),
            token: self.previous.clone(),
        })
    }
}
