use crate::{Token, compiler::{Expr, WovenExpr, symbol_table::SymbolKind, weaves::Weave}, weave_analyser::WeaveAnalyzer};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_assign_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        name: Token,
        value: Expr,
    ) -> crate::weave_analyser::WeaveResult<WovenExpr> {
        if let Some(resolved) = self.symbol_table.resolve(&name.lexeme).cloned() {
                    match *resolved.kind.borrow() {
                        SymbolKind::Variable { mutable } => {
                            if !mutable {
                                self.error(
                            "Tried to reassign a value to a 'bind'. Binds cannot be reassigned!",
                            name,
                        );
                                return Ok(WovenExpr::Cursed { span: None });
                            }
                        }
                        _ => {
                            self.error("The value isnt a variable!", name);
                            return Ok(WovenExpr::Cursed { span: None });
                        }
                    };

                    let woven_expr = self.analyze_expression(value, None)?;
                    let weave = woven_expr.weave();

                    if let Some(expected_weave) = expected_weave {
                        if *expected_weave != weave {
                            self.error(
                                &format!(
                                    "The weave of the value being assigned does not match the expected weave '{}'",
                                    expected_weave.get_name()
                                ),
                                name,
                            );
                            return Ok(WovenExpr::Cursed { span: None });
                        }
                    }

                    // Assignment requires an exact match of the tapestry!
                    if resolved.weave == woven_expr.weave() {
                        return Ok(WovenExpr::Assignment {
                            name: name,
                            value: Box::new(woven_expr),
                            weave: weave,
                            symbol: resolved,
                        });
                    }

                    self.error(
                        "The assignee and the value to be assigned are of different Weaves!\nAssignment failed.",
                        name,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                } else {
                    self.error(
                        "The mark was no where to be found from this realm!\nVariable resolution failed.",
                        name,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }
    }
}