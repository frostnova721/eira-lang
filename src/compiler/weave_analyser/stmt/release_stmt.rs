use crate::{
    Token, compiler::{Expr, WovenStmt, symbol_table::SymbolKind, weaves::Weave}, weave_analyser::{Realm, WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_release_stmt(
        &mut self,
        token: Token,
        expr: Option<Expr>,
    ) -> WeaveResult<WovenStmt> {
        // Ensure 'release' is only used within a spell realm
        if self.current_realm == Realm::Genesis {
            self.error(
                "Values cannot be released from the 'Genesis' realm!\n\
                        Error: Usage of 'release' outside the spell scope.",
                token,
            );
            return Ok(WovenStmt::Cursed { span: None });
        }

        let curr_spell_name = match self.spell_stack.last() {
            Some(name) => name.clone(),
            None => {
                self.error("Release used outside of any spell scope.", token);
                return Ok(WovenStmt::Cursed { span: None });
            }
        };

        // Ensure spell exists and check if already released
        let spell_entry = match self.symbol_table.resolve(&curr_spell_name).cloned() {
            Some(v) => {
                let kind = v.kind.borrow().clone();
                match kind {
                    SymbolKind::Spell(info) => info,
                    _ => {
                        self.error(
                            &format!(
                                "No Spell found in the realm with the name '{}'",
                                curr_spell_name
                            ),
                            token,
                        );
                        return Ok(WovenStmt::Cursed { span: None });
                    }
                }
            }
            None => {
                self.error(
                    &format!(
                        "No Spell found in the realm with the name '{}'",
                        curr_spell_name
                    ),
                    token,
                );
                return Ok(WovenStmt::Cursed { span: None });
            }
        };

        let expected_weave = spell_entry.release_weave.clone();

        if let Some(e) = expr {
            let w_expr = self.analyze_expression(e, Some(&expected_weave))?;

            // Try to get the weave from the symbol first (for variables with composite weaves)
            // Otherwise fall back to tapestry lookup
            let actual_weave = if let Some(symbol) = w_expr.symbol() {
                symbol.weave.clone()
            } else {
                w_expr.weave()
            };

            // Exact tapestry check (spells should return the exact weave)
            match &expected_weave {
                Weave::Maybe(inner) => {
                    if actual_weave == Weave::Empty || actual_weave == **inner {
                        // valid release
                    } else {
                        self.error(
                            &format!(
                                "The spell '{}' was expected to release '{}' but '{}' was released",
                                curr_spell_name,
                                expected_weave.get_name(),
                                actual_weave.get_name()
                            ),
                            token,
                        );
                        return Ok(WovenStmt::Cursed { span: None });
                    }
                }
                _ => {
                    if expected_weave != actual_weave {
                        self.error(
                            &format!(
                                "The spell '{}' was expected to release '{}' but '{}' was released",
                                curr_spell_name,
                                expected_weave.get_name(),
                                actual_weave.get_name()
                            ),
                            token,
                        );
                        return Ok(WovenStmt::Cursed { span: None });
                    }
                }
            }

            Ok(WovenStmt::Release {
                token: token,
                expr: Some(w_expr),
            })
        } else {
            // release; with no expression implies Emptiness.
            // If the spell expects a non-empty weave, this is an error.
            if expected_weave != Weave::Empty {
                self.error(
                            &format!(
                                "The spell '{}' expects a value of weave '{}' to be released, but no value was provided.",
                                curr_spell_name, expected_weave.get_name()
                            ),
                            token,
                        );
                return Ok(WovenStmt::Cursed { span: None });
            }

            Ok(WovenStmt::Release {
                token: token,
                expr: None,
            })
        }
    }
}
