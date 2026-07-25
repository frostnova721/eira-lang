use crate::{Token, compiler::{
    Expr, WovenExpr, strand::{ADDITIVE_STRAND, CONCATINABLE_STRAND}, token_type::TokenType, weaves::Weave,
}, weave_analyser::{WeaveAnalyzer, WeaveResult}};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_binary_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        left: Expr,
        right: Expr,
        operator: Token,
    ) -> WeaveResult<WovenExpr> {
        let w_left = self.analyze_expression(left, None)?;
        let w_right = self.analyze_expression(right, None)?;

        if operator.token_type == TokenType::Plus {
            let left_has_additive = w_left.weave().get_tapestry().has_strand(ADDITIVE_STRAND);
            let left_has_concat = w_left
                .weave()
                .get_tapestry()
                .has_strand(CONCATINABLE_STRAND);
            let right_has_additive = w_right.weave().get_tapestry().has_strand(ADDITIVE_STRAND);
            let right_has_concat = w_right
                .weave()
                .get_tapestry()
                .has_strand(CONCATINABLE_STRAND);

            // Both must support the same type of operation
            if (left_has_additive && right_has_additive) || (left_has_concat && right_has_concat) {
                // Valid operation
            } else {
                self.error(
                            "Cannot perform '+' operation: operands must both contain either 'Additive' or 'Concatinable' strand.",
                            operator,
                        );
                return Ok(WovenExpr::Cursed { span: None });
            }
        } else {
            if let Some(req_strand) = self.strand_from_op(operator.token_type) {
                if !w_left.weave().get_tapestry().has_strand(req_strand) {
                    self.error(
                        &format!(
                            "The weave of one of the operands is not composed of {} strand.",
                            self.strand_string_from_bits(req_strand)
                        ),
                        operator,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }

                if !w_right.weave().get_tapestry().has_strand(req_strand) {
                    self.error(
                        &format!(
                            "The weave of one of the operands is not composed of {} strand.",
                            self.strand_string_from_bits(req_strand)
                        ),
                        operator,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            } else {
                self.error(
                    &format!("Unknown operation '{}'", operator.lexeme),
                    operator,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        }

        let result_weave = match operator.token_type {
            TokenType::Greater
            | TokenType::Less
            | TokenType::EqualEqual
            | TokenType::LessEqual
            | TokenType::GreaterEqual
            | TokenType::BangEqual => Weave::Truth,
            TokenType::Plus => {
                // hard coded for now. Should be dynamic later
                if w_left.weave().get_tapestry().has_strand(ADDITIVE_STRAND)
                    && w_right.weave().get_tapestry().has_strand(ADDITIVE_STRAND)
                {
                    Weave::Num
                } else {
                    Weave::Text
                }
            }
            _ => w_left.weave(), // Assumes left-hand side's type
        };

        if let Some(expected) = expected_weave {
            if *expected != result_weave {
                self.error(
                            &format!(
                                "The result weave of the binary operation '{}' does not match the expected weave '{}'",
                                operator.lexeme,
                                expected.get_name()
                            ),
                            operator,
                        );
                return Ok(WovenExpr::Cursed { span: None });
            }
        }

        Ok(WovenExpr::Binary {
            left: Box::new(w_left),
            right: Box::new(w_right),
            operator: operator,
            weave: result_weave,
        })
    }
}
