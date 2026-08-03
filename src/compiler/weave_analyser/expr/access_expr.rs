use std::cell::RefCell;

use crate::{
    compiler::{
        Expr, WovenExpr,
        symbol_table::{Symbol, SymbolKind},
        token_type::TokenType,
        types::Visibility,
        weaves::Weave,
    },
    values::native_spell::NativeSpell,
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_access_expr(
        &mut self,
        _expected_weave: Option<&Weave>,
        expr: Expr,
    ) -> WeaveResult<WovenExpr> {
        let (is_safe_access, material, property) = match expr {
            Expr::Access { material, property } => (false, material, property),
            Expr::SafeAccess { material, property } => (true, material, property),
            _ => unreachable!(),
        };

        let w_material = self.analyze_expression(*material, None)?;
        if let Weave::Module(module_name) = w_material.weave() {
            if is_safe_access {
                self.error(
                    "You don't have to use '?.' for accessing tethered scrolls.",
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }

            let symbol = match w_material {
                WovenExpr::Variable { ref symbol, .. } => symbol.clone(),
                _ => {
                    self.error(&format!("{} is not a mark or a bind", property), property);
                    return Ok(WovenExpr::Cursed { span: None });
                }
            };

            let mod_info = symbol.kind.borrow();

            let Some(module) = mod_info.get_module_info() else {
                // this shouldnt be thrown (if im not wrong)
                self.error(
                    &format!("The scroll tethered to '{}' doesn't exist.", module_name),
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            };

            let Some(sym) = module.get(&property.lexeme) else {
                self.error(
                    &format!(
                        "The symbol '{}' cannot be found in the scroll '{}'",
                        property.lexeme, module_name
                    ),
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            };

            if sym.visibility == Visibility::Secret {
                self.error(
                    &format!(
                        "The '{}' is currently a secret inside the scroll!",
                        property.lexeme
                    ),
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }

            // return match sym.kind.borrow().clone() {
            // SymbolKind::Variable { mutable } =>
            return Ok(WovenExpr::Variable {
                name: property.clone(),
                weave: sym.weave.clone(),
                symbol: sym.clone(),
            });
            // SymbolKind::Spell(spell_info) => Ok(WovenExpr::BoundSpell {
            //     is_safe: false,
            //     material: Box::new(w_material),
            //     spell_symbol: sym.clone(),
            //     token: property.clone(),
            //     weave: spell_info.release_weave,
            // }),
            // _ => {
            // return self.error("h,", property);
            // }
            // };
        }

        let (target, is_primitive) = match (is_safe_access, w_material.weave()) {
            (false, Weave::Sign(s)) => (s, false),
            (true, Weave::Maybe(inner)) => {
                if let Weave::Sign(s) = *inner {
                    (s, false)
                } else {
                    (inner.get_base_name(), true)
                    // self.error(
                    //     &format!(
                    //         "Only signs can be accessed with '.' operator! Got {}.",
                    //         inner.get_name()
                    //     ),
                    //     property,
                    // );
                    // return Ok(WovenExpr::Cursed { span: None });
                }
            }
            (true, _) => {
                self.error(
                    "Safe access operation (?.) is only possible for Maybe<W>.",
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }

            (false, w) => {
                (w.get_base_name(), true)
                // self.error(
                //     &format!(
                //         "Only signs can be accessed with '.' operator! Got {}.",
                //         w.get_name()
                //     ),
                //     property,
                // );
                // return Ok(WovenExpr::Cursed { span: None });
            }
        };

        if is_primitive {
            let global_name = format!("core:{}:{}", target, property.lexeme);

            let Ok(spell) = NativeSpell::resolve_methods(&global_name, w_material.weave()) else {
                self.error(
                    &format!(
                        "The seal or spell '{}' is not defined for '{}' weave!",
                        property.lexeme, target
                    ),
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            };

            let spell_info = match NativeSpell::get_spell_info(spell) {
                Ok(info) => info,
                Err(_) => {
                    self.error(
                        &format!(
                            "The seal or spell '{}' is not defined for '{}' weave!",
                            property.lexeme, target
                        ),
                        property,
                    );

                    return Ok(WovenExpr::Cursed { span: None });
                }
            };

            let spell_sym = Symbol {
                name: global_name.clone(),
                weave: Weave::Spell {
                    release: Box::new(spell_info.release_weave.clone()),
                },
                depth: 0,
                kind: RefCell::new(SymbolKind::Spell(spell_info.clone())),
                slot_idx: 0,
                parent: None,
                visibility: Visibility::Public,
            };

            return Ok(WovenExpr::BoundSpell {
                is_safe: is_safe_access,
                material: Box::new(w_material),
                spell_symbol: spell_sym.clone(),
                token: property.clone(),
                weave: spell_sym.weave.clone(),
            });
        }

        // wether the material passed is the defined name of sign
        let is_declared_symbol = target == w_material.token().lexeme.as_str();
        // let w_material = self.analyze_expression(*material, None)?;
        // // it should be a variable expression
        // let sign_name = match w_material.weave() {
        //     Weave::Sign(s) => s,
        //     _ => {
        //         return self
        //             .error("Only signs can be accessed with '.' operator!", property);
        //     }
        // };

        let Some(sign_symbol) = self.symbol_table.resolve(&target) else {
            self.error(
                &format!(
                    "The sign '{}' was not found across the eira realms!",
                    target
                ),
                property,
            );
            return Ok(WovenExpr::Cursed { span: None });
        };

        let Some(sign_info) = sign_symbol.kind.borrow().get_sign_info() else {
            self.error(&format!("'{}' is not a sign!", sign_symbol.name), property);
            return Ok(WovenExpr::Cursed { span: None });
        };

        if let Some(mark) = sign_info.schema.get_field_index(property.lexeme.clone()) {
            let Some(property_weave) = sign_info.marks.get(&property.lexeme) else {
                self.error(
                    &format!(
                        "Eira couldn't find the weave for property '{}'",
                        property.lexeme
                    ),
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            };

            if is_safe_access {
                return Ok(WovenExpr::SafeAccess {
                    material: Box::new(w_material),
                    property,
                    field_name_idx: mark as u16,
                    weave: Weave::Maybe(Box::new(property_weave.clone())),
                });
            }

            return Ok(WovenExpr::Access {
                material: Box::new(w_material),
                property,
                field_name_idx: mark as u16,
                weave: property_weave.clone(),
            });
        }

        if let Some(attunement) = sign_info.attunements.get(&property.lexeme) {
            if is_declared_symbol && !attunement.is_static {
                self.error(
                    "Attunements cannot be invoked directly from the sign!",
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            } else if !is_declared_symbol && attunement.is_static {
                self.error(
                    "Static attunements can only be invoked directly from the sign!",
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }

            if attunement.visibility == Visibility::Secret
                && w_material.token().token_type != TokenType::Ego
            {
                self.error(
                                &format!(
                                    "The spell '{}' attuned to sign '{}' is a secret and cannot be casted here!",
                                    attunement.method_name, target,
                                ),
                                property,
                            );
                return Ok(WovenExpr::Cursed { span: None });
            }

            let spell_symbol = match self.symbol_table.resolve(&attunement.method_name) {
                Some(s) => s.clone(),
                None => {
                    self.error(
                        &format!(
                            "The spell '{}' was not found for sign '{}'!",
                            attunement.method_name, target
                        ),
                        property,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            };

            return Ok(WovenExpr::BoundSpell {
                is_safe: is_safe_access,
                material: Box::new(w_material),
                spell_symbol: spell_symbol.clone(),
                token: property.clone(),
                weave: spell_symbol.weave.clone(),
            });
        }

        self.error(
            &format!(
                "The mark or spell '{}' is not defined for '{}'",
                property.lexeme, target
            ),
            property,
        );
        return Ok(WovenExpr::Cursed { span: None });
    }
}
