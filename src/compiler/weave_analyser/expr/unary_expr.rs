use crate::{
    Token,
    compiler::{Expr, WovenExpr, token_type::TokenType, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_unary_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        operand: Expr,
        operator: Token,
    ) -> WeaveResult<WovenExpr> {
        if operator.token_type != TokenType::Minus && operator.token_type != TokenType::Bang {
            self.error("Unknown Unary Operation", operator);
            return Ok(WovenExpr::Cursed { span: None });
        }
        if let Some(strand) = self.strand_from_op(operator.token_type) {
            let expr = self.analyze_expression(operand, None)?;
            if !expr.weave().get_tapestry().has_strand(strand) {
                self.error(
                            &format!(
                                "The operand does not contain the '{}' strand as required by '{}' operation",
                                self.strand_string_from_bits(strand),
                                operator.lexeme
                            ),
                            operator,
                        );
                return Ok(WovenExpr::Cursed { span: None });
            }
            let weave = expr.weave();

            if let Some(expected) = expected_weave {
                if *expected != weave {
                    self.error(
                                &format!(
                                    "The result weave of the unary operation '{}' does not match the expected weave '{}'",
                                    operator.lexeme,
                                    expected.get_name()
                                ),
                                operator,
                            );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            }

            Ok(WovenExpr::Unary {
                operand: Box::new(expr),
                operator: operator,
                weave: weave,
            })
        } else {
            self.error("Unknown Operation", operator);
            return Ok(WovenExpr::Cursed { span: None });
        }
    }
}
