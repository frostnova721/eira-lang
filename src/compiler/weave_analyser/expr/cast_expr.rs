use std::rc::Rc;

use crate::{
    Token,
    compiler::{Expr, WovenExpr, strand::CALLABLE_STRAND, symbol_table::SymbolKind, weaves::Weave},
    values::{native_spell::NativeSpell, spell::SpellInfo},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_cast_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        reagents: Vec<Expr>,
        callee: Expr,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
        let callee_token = match &callee {
            Expr::Variable { name } => name.clone(),
            _ => token.clone(),
        };

        if let Expr::Variable { name } = &callee {
            if let Some(native_spell) = NativeSpell::resolve(&name.lexeme).ok() {
                let native_info = NativeSpell::get_spell_info(native_spell.clone()).unwrap();

                if native_info.reagents.len() != reagents.len() {
                    self.error(
                        &format!(
                            "The spell '{}' expected {} reagents, but you provided {} of them!",
                            native_info.name,
                            native_info.reagents.len(),
                            reagents.len()
                        ),
                        callee_token.clone(),
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }

                if let Some(expected) = expected_weave {
                    if *expected != native_info.release_weave {
                        self.error(
                            &format!(
                                "The release weave of spell '{}' does not match the expected weave '{}'",
                                native_info.name,
                                expected.get_name()
                            ),
                            callee_token.clone(),
                        );
                        return Ok(WovenExpr::Cursed { span: None });
                    }
                }

                let mut w_reagents: Vec<WovenExpr> = vec![];
                for (i, reagent) in reagents.iter().enumerate() {
                    let expected = native_info.reagents.get(i).unwrap();
                    let w_expr = self.analyze_expression(reagent.clone(), Some(&expected.weave))?;
                    if w_expr.weave() != expected.weave {
                        self.error(
                            &format!(
                                "The reagent #{} was expected to be {}, but got {}",
                                i + 1,
                                expected.weave.get_name(),
                                w_expr.weave().get_name()
                            ),
                            callee_token.clone(),
                        );
                        return Ok(WovenExpr::Cursed { span: None });
                    }
                    w_reagents.push(w_expr.clone());
                }

                return Ok(WovenExpr::NativeCast {
                    reagents: w_reagents,
                    callee: callee_token,
                    weave: native_info.release_weave,
                    native_spell,
                });
            }
        }

        let w_callee = self.analyze_expression(callee, None);

        if let Ok(WovenExpr::BoundSpell {
            is_safe,
            material,
            spell_symbol,
            token,
            weave: _,
        }) = &w_callee
        {
            let method_symbol = spell_symbol;
            self.resolve_n_add_upvalue(&method_symbol)?;
            let spell_info = method_symbol.kind.borrow().get_spell_info().unwrap();

            if let Some(expected) = expected_weave {
                if *expected != spell_info.release_weave {
                    self.error(
                                    &format!(
                                        "The release weave of spell '{}' does not match the expected weave '{}'",
                                        method_symbol.name,
                                        expected.get_name()
                                    ),
                                    token.clone(),
                                );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            }

            let mut final_reagents = vec![*material.clone()];

            for (i, r) in reagents.iter().enumerate() {
                let w_r = self.analyze_expression(
                    r.clone(),
                    Some(&spell_info.reagents.get(i+1).unwrap().weave),
                )?;
                final_reagents.push(w_r);
            }

            if final_reagents.len() != spell_info.reagents.len() {
                self.error(
                    &format!(
                        "The spell '{}' expected {} reagent(s), but you provided {} of them!",
                        method_symbol.name,
                        spell_info.reagents.len().saturating_sub(1), // one is ego
                        final_reagents.len().saturating_sub(1)
                    ),
                    token.clone(),
                );
                return Ok(WovenExpr::Cursed { span: None });
            }

            if let Ok(native_spell) =
                NativeSpell::resolve_methods(&spell_symbol.name, material.weave())
            {
                return Ok(WovenExpr::NativeCast {
                    reagents: final_reagents,
                    callee: token.clone(),
                    weave: spell_info.release_weave.clone(),
                    native_spell,
                });
            }

            if *is_safe {
                return Ok(WovenExpr::SafeCast {
                    callee: token.clone(),
                    reagents: final_reagents,
                    spell_symbol: method_symbol.clone(),
                    weave: spell_info.release_weave.clone(),
                });
            } else {
                return Ok(WovenExpr::Cast {
                    callee: token.clone(),
                    reagents: final_reagents,
                    spell_symbol: method_symbol.clone(),
                    weave: spell_info.release_weave.clone(),
                });
            }
            // } else {
            // return self.error(
            // "for now... just be satisfied with spell casting only on signs!",
            // w_material.token(),
            // );
            // }
        }

        let native = match &w_callee {
            Err(_) => {
                let nat = NativeSpell::resolve(&token.lexeme);

                if nat.is_ok() {
                    Some(nat.unwrap())
                } else {
                    None
                }
            }
            Ok(_) => None,
        };

        if native.is_some() {
            let native_spell = native.unwrap();

            let native_info = NativeSpell::get_spell_info(native_spell.clone()).unwrap();

            if native_info.reagents.len() != reagents.len() {
                self.error(
                    &format!(
                        "The spell '{}' expected {} reagents, but you provided {} of them!",
                        native_info.name,
                        native_info.reagents.len(),
                        reagents.len()
                    ),
                    token,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }

            if let Some(expected) = expected_weave {
                if *expected != native_info.release_weave {
                    self.error(
                                    &format!(
                                        "The release weave of spell '{}' does not match the expected weave '{}'",
                                        native_info.name,
                                        expected.get_name()
                                    ),
                                    token,
                                );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            }

            let mut w_reagents: Vec<WovenExpr> = vec![];
            for (i, reagent) in reagents.iter().enumerate() {
                let expected = native_info.reagents.get(i).unwrap();
                let w_expr = self.analyze_expression(reagent.clone(), Some(&expected.weave))?;
                if w_expr.weave() != expected.weave {
                    self.error(
                        &format!(
                            "The reagent #{} was expected to be {}, but got {}",
                            i + 1,
                            expected.weave.get_name(),
                            w_expr.weave().get_name()
                        ),
                        token,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }
                w_reagents.push(w_expr.clone());
            }

            return Ok(WovenExpr::NativeCast {
                reagents: w_reagents,
                callee: token,
                weave: native_info.release_weave,
                native_spell,
            });
        }

        let w_callee = w_callee?;

        if !w_callee.weave().get_tapestry().has_strand(CALLABLE_STRAND) {
            self.error(
                        "Cannot perform cast on a compile-time unknown spell. Only direct sign method calls are allowed to be casted for now.",
                        token,
                    );
            return Ok(WovenExpr::Cursed { span: None });
        }

        // atp its usually a variable expr. If its not, well... good luck ig
        let (spell_info, spell_symbol) = match w_callee {
            WovenExpr::Variable { symbol, .. } => {
                if let SymbolKind::Spell(si) = &*symbol.kind.borrow() {
                    (si.clone(), symbol.clone())
                } else {
                    let mut spell_info: Option<SpellInfo> = None;
                    let mut s = symbol.clone();
                    while let Some(p) = s.parent {
                        if let SymbolKind::Spell(si) = &*p.kind.borrow() {
                            spell_info = Some(si.clone());
                            break;
                        }
                        s = Rc::unwrap_or_clone(p);
                    }

                    match spell_info {
                        Some(si) => (si, symbol),

                        // if not found, try checking Native Spells
                        None => {
                            self.error("Only spells can be casted!", token);
                            return Ok(WovenExpr::Cursed { span: None });
                        }
                    }
                }
            }
            _ => {
                // this should be unreachable.. if im not wrong
                self.error("Eira can only cast a spell from a variable!", token);
                return Ok(WovenExpr::Cursed { span: None });
            }
        };

        if reagents.len() != spell_info.reagents.len() {
            self.error(
                &format!(
                    "The spell '{}' expected {} reagent(s), but you provided {} of them!",
                    spell_info.name,
                    spell_info.reagents.len(),
                    reagents.len()
                ),
                token,
            );
            return Ok(WovenExpr::Cursed { span: None });
        }

        if let Some(expected) = expected_weave {
            if *expected != spell_info.release_weave {
                self.error(
                    &format!(
                        "The release weave of spell '{}' does not match the expected weave '{}'",
                        spell_info.name,
                        expected.get_name()
                    ),
                    token,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        }

        let mut final_reagents: Vec<WovenExpr> = vec![];

        for r in reagents {
            final_reagents.push(self.analyze_expression(r, None)?);
        }

        // let Some(spell_symbol) = self.symbol_table.resolve(&spell_info.name) else {
        //     return self.error(
        //         &format!("Spell symbol not found while casting! for {}", token),
        //         token,
        //     );
        // };

        Ok(WovenExpr::Cast {
            callee: token.clone(),
            reagents: final_reagents,
            spell_symbol: spell_symbol,
            weave: spell_info.release_weave,
        })
    }
}
