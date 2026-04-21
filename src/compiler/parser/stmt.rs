use crate::{Parser, compiler::{Stmt, parser::{parser::MSG_MISSED_SEMICOLON, types::ParseResult}, token_type::TokenType}};

impl Parser {
      pub(super) fn block(&mut self) -> ParseResult<Stmt> {
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

    pub(super) fn release_statement(&mut self) -> ParseResult<Stmt> {
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

    pub(super) fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let e = self.expression()?;
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::ExprStmt { expr: e })
    }

    pub(super) fn chant_statment(&mut self) -> ParseResult<Stmt> {
        let exp = self.expression()?;
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::Chant { expression: exp })
    }

    pub(super) fn while_statement(&mut self) -> ParseResult<Stmt> {
        let condition = self.expression()?;
        self.consume(TokenType::BraceLeft, "Expected '{' after loop condition.");
        let body = self.block()?;
        Ok(Stmt::While {
            condition: condition,
            body: Box::new(body),
        })
    }

    pub(super) fn fate_statement(&mut self) -> ParseResult<Stmt> {
        let condition = self.expression()?;
        self.consume(TokenType::BraceLeft, "Expected '{' at start of fate block.");
        let then_branch = self.block()?;

        let else_branch = if self.match_token(TokenType::Divert) {
            if self.match_token(TokenType::Fate) {
                Some(Box::new(self.fate_statement()?))
            } else {
            self.consume(
                TokenType::BraceLeft,
                "Expected '{' at start of fate-else block.",
            );
            Some(Box::new(self.block()?))
        }
        } else {
            None
        };
        Ok(Stmt::Fate {
            condition: condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch,
        })
    }

    pub(super) fn sever_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::Sever {
            token: self.previous.clone(),
        })
    }

    pub(super) fn flow_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::SemiColon, MSG_MISSED_SEMICOLON);
        Ok(Stmt::Flow {
            token: self.previous.clone(),
        })
    }
}
