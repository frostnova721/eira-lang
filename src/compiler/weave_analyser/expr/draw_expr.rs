use crate::{
    Token, Value, compiler::{Expr, WovenExpr, mark::{EtchedMark, WovenEtchedMark}, weaves::Weave}, weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_draw_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        marks: Vec<EtchedMark>,
        callee: Token,
    ) -> WeaveResult<WovenExpr> {
        let Some(symbol) = self.symbol_table.resolve(&callee.lexeme).cloned() else {
            self.error(
                &format!("The sign '{}' was not found!", callee.lexeme),
                callee,
            );
            return Ok(WovenExpr::Cursed { span: None });
        };

        let sign_info = {
            let Some(info) = symbol.kind.borrow().get_sign_info() else {
                self.error(&format!("'{}' is not a sign!", symbol.name), callee);
                return Ok(WovenExpr::Cursed { span: None });
            };

            info.clone()
        };

        // Will have to change for optional fields
        if sign_info.marks.len() != marks.len() {
            self.error(
                &format!(
                    "The sign '{}' expected {} marks, but you provided{} {} of them!",
                    callee.lexeme,
                    sign_info.marks.len(),
                    if marks.len() < sign_info.marks.len() {
                        " only"
                    } else {
                        ""
                    },
                    marks.len()
                ),
                callee,
            );
            return Ok(WovenExpr::Cursed { span: None });
        }

        let mut w_marks: Vec<WovenEtchedMark> = vec![];
        for mark in marks {
            if let Some(field) = sign_info.marks.get(&mark.name.lexeme) {
                // set blank as a way to set empty value
                let mark_val = match mark.expr {
                    Expr::Blank { token } => WovenExpr::Literal {
                        value: Value::Emptiness,
                        token: token,
                        weave: Weave::Empty,
                    },
                    _ => self.analyze_expression(mark.expr, None)?,
                };

                let mark_weave = mark_val.weave();
                if self.can_assign(field, &mark_weave) {
                    w_marks.push(WovenEtchedMark {
                        name: mark.name.clone(),
                        expr: mark_val.clone(),
                    })
                } else {
                    self.error(
                        &format!(
                            "The mark '{}' was expected to have weave '{}' but got '{}'",
                            mark.name.lexeme,
                            field.get_name(),
                            mark_weave.get_name()
                        ),
                        mark.name,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            } else {
                self.error(
                    &format!(
                        "The mark '{}' doesn't exist inside {}",
                        mark.name.lexeme, callee.lexeme
                    ),
                    mark.name,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        }

        Ok(WovenExpr::Draw {
            marks: w_marks,
            callee: callee.clone(),
            weave: Weave::Sign(sign_info.schema.name.clone()),
            sign_symbol: symbol.clone(),
        })
    }
}
