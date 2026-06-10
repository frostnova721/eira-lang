use crate::{
    Parser, Token,
    compiler::{
        Expr, Stmt,
        mark::Mark,
        parser::{
            parser::{MSG_BIND_VALUE_NOT_INITIALIZED, MSG_MISSED_SEMICOLON},
            types::{ParseResult, ParsedWeave},
        },
        reagents::Reagent,
        token_type::TokenType,
    },
};

impl Parser {
    pub(super) fn spell_declaration(&mut self, attuned_to: Option<Token>) -> ParseResult<Stmt> {
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
                let weave = self.parse_weave("Expected a weave name after ';'")?;

                params.push(Reagent {
                    name: token,
                    weave: weave,
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
            attuned_to,
        })
    }

    pub(super) fn variable_declaration(&mut self, mutable: bool) -> ParseResult<Stmt> {
        self.consume(TokenType::Identifier, "Expected a variable name!");
        let name = self.previous.clone();
        let initializer: Option<Expr>;

        let mut weave: Option<ParsedWeave> = None;

        if self.match_token(TokenType::Colon) {
            weave = Some(self.parse_weave("Expected a weave name to bind with the variable!")?);
        }

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
            weave: weave,
        })
    }

    pub(super) fn sign_declaration(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::Identifier, "Expected a name for the sign.");
        let name = self.previous.clone();
        self.consume(TokenType::BraceLeft, "Expected '{' after the sign name.");

        let mut marks: Vec<Mark> = vec![];

        if !self.check(TokenType::BraceRight) {
            loop {
                if self.match_token(TokenType::Identifier) {
                    let mark_name = self.previous.clone();
                    self.consume(
                        TokenType::Colon,
                        "Expected a weave definition for the field.",
                    );

                    let parsed_weave =
                        self.parse_weave("Expected a weave name to bind with the sign's mark!")?;
                    marks.push(Mark {
                        name: mark_name,
                        parsed_weave: parsed_weave,
                    });

                    if self.match_token(TokenType::Comma) {
                        continue;
                    } else {
                        break; // break out of the loop if no comma is found, meaning the end of the mark list!
                    }
                } else {
                    break; // simply break out of the loop, any errors should be caught by following codes (hopefully)
                }
            }
        }

        self.consume(TokenType::BraceRight, "Expected '}' after sign marks.");
        // self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);

        Ok(Stmt::Sign { name, marks })
    }

    pub(super) fn attune_declaration(&mut self) -> ParseResult<Stmt> {
        self.consume(
            TokenType::Identifier,
            "Expected a name for the sign to attune to.",
        );
        let sign = self.previous.clone();

        self.consume(TokenType::BraceLeft, "Expected '{' after the sign name.");

        let mut spells: Vec<Box<Stmt>> = vec![];

        while !self.check(TokenType::BraceRight) && !self.reached_end() {
            if self.match_token(TokenType::Spell) {
                spells.push(Box::new(self.spell_declaration(Some(sign.clone()))?));
            } else {
                // println!("Hit else in attune! Current token: {:?}", self.current.token_type);
                self.throw_error("Only spell declarations are allowed in an attunement block!");
                break;
            }
        }

        self.consume(
            TokenType::BraceRight,
            "Expected '}' after the attunement block.",
        );

        Ok(Stmt::Attune { sign, spells })
    }

    pub(super) fn tether_declaration(&mut self) -> ParseResult<Stmt> {
        let token = self.previous.clone();

        let mut is_path = false;

        let path = if self.match_token(TokenType::String) {
            is_path = true;
            vec![self.previous.clone()]
        } else {
            let mut joined_path: Vec<Token> = vec![];
            loop {
                self.consume(TokenType::Identifier, "Expected an identifier.");
                joined_path.push(self.previous.clone());
                if self.match_token(TokenType::Dot) {
                    continue;
                } else {
                    break;
                }
            }
            joined_path
        };

        let bind_to = if self.match_token(TokenType::Bind) {
            self.consume(
                TokenType::Identifier,
                "Expected an identifier after 'bind'.",
            );
            Some(self.previous.clone())
        } else {
            None
        };

        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);

        Ok(Stmt::Tether {
            token,
            path: path,
            bind_to: bind_to,
            is_path: is_path,
        })
    }
}
