use std::cell::RefCell;

use crate::{
    Token,
    compiler::{
        Stmt,
        ast::decl::WovenDecl,
        parser::types::ParsedWeave,
        reagents::{Reagent, WovenReagent},
        symbol_table::SymbolKind,
        types::Visibility,
        weaves::Weave,
    },
    values::{sign::AttunedSpell, spell::SpellInfo},
    weave_analyser::{Realm, WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_spell(
        &mut self,
        name: Token,
        reagents: Vec<Reagent>,
        visibility: Option<Visibility>,
        return_weave: Option<ParsedWeave>,
        attuned_to: Option<Token>,
        body: Box<Stmt>,
    ) -> WeaveResult<WovenDecl> {
        // allow spell shadowing from outer scopes
        let existing = self.symbol_table.resolve_in_current_scope(&name.lexeme);
        if existing.is_some() {
            return self.error(
                &format!(
                    "The spell '{}' already exists in the current scope!",
                    name.lexeme
                ),
                name,
            );
        }

        // set public visibility by default
        let visibility = visibility.unwrap_or(Visibility::default());

        let mut w_reagents: Vec<WovenReagent> = vec![];
        let slot = self.symbol_table.get_current_scope_size();

        // get the ret type (weave ofcourse)
        let ret_weave = match return_weave {
            Some(rw) => self.analyze_parsed_weave(rw)?,
            None => Weave::Empty,
        };

        // define the spell
        // Create SpellWeave<ReturnWeave> for the spell's symbol
        let spell_weave = Weave::Spell {
            // reagents: Vec::new(),
            release: Box::new(ret_weave.clone()),
        };

        let spell_name = match attuned_to.as_ref() {
            Some(attuned) => format!("{}:{}", attuned.lexeme, name.lexeme),
            None => name.lexeme.clone(),
        };

        // mark the symbol definition
        let mut stub_symbol = self
            .symbol_table
            .define_spell(
                spell_name.clone(),
                spell_weave.clone(),
                SpellInfo {
                    name: spell_name.clone(),
                    reagents: w_reagents.clone(),
                    release_weave: ret_weave.clone(),
                    upvalues: vec![],
                },
                slot,
                None,
                visibility.clone(),
            )
            .unwrap(); // this shouldmt be failing

        self.symbol_table.new_scope();

        // spell_base_depth should be equal to depth where spell is defined;
        // so the base_depth should be incremented after savin it
        // Variables from this depth or shallower can be upvalues
        let saved_spell_base_depth = self.spell_base_depth;
        self.spell_base_depth = self.symbol_table.get_depth() - 1;

        // Reset spell slot counter for parameters
        self.spell_slot_counter = 0;

        let upvals_saved = std::mem::take(&mut self.current_upvalues);

        if let Some(sign) = attuned_to {
            let sign_lexeme = &sign.lexeme;
            let name_lexeme = &name.lexeme;
            let method_name = format!("{}:{}", sign_lexeme, name_lexeme);

            {
                let Some(s) = self.symbol_table.resolve(sign_lexeme) else {
                    return self.error(
                        &format!(
                            "No symbol found across the eira realms with the name '{}'.",
                            sign_lexeme
                        ),
                        sign.clone(),
                    );
                };

                let mut kind = s.kind.borrow_mut();

                match &mut *kind {
                    SymbolKind::Sign(si) => {
                        if si.attunements.contains_key(&name.lexeme) {
                            return self.error(&format!("The sign '{}' is already attuned to a spell named '{}', Try renaming the spell.",sign_lexeme, name_lexeme),
                                    sign.clone(),);
                        }

                        let attuned = AttunedSpell {
                            method_name: method_name.clone(),
                            visibility: visibility.clone(),
                            is_static: false,
                        };

                        si.attunements.insert(name_lexeme.clone(), attuned);
                    }
                    _ => {
                        return self.error(
                            &format!(
                                "'{}' is not a sign. Attunement can only be done on signs.",
                                sign_lexeme
                            ),
                            sign.clone(),
                        );
                    }
                }
            }

            self.symbol_table.define_variable(
                "ego".to_string(),
                Weave::Sign(sign_lexeme.clone()),
                false,
                self.spell_slot_counter,
                None,
                Visibility::Secret,
            );
            self.spell_slot_counter += 1;

            w_reagents.push(WovenReagent {
                weave: Weave::Sign(sign_lexeme.clone()),
            });
        }

        for r in reagents {
            let weave = self.analyze_parsed_weave(r.weave)?;
            self.symbol_table.define_variable(
                r.name.lexeme.clone(),
                weave.clone(),
                false,
                self.spell_slot_counter, // Use continuous slot counter, (lexical scoping doesnt work right here!)
                None,
                Visibility::Secret,
            );
            self.spell_slot_counter += 1; // Increment for next parameter
            w_reagents.push(WovenReagent {
                // name: r.name.clone(),
                weave: weave,
            });
        }

        stub_symbol.kind = RefCell::new(SymbolKind::Spell(SpellInfo {
            name: stub_symbol.name.clone(),
            reagents: w_reagents.clone(),
            release_weave: ret_weave.clone(),
            upvalues: vec![],
        }));

        self.symbol_table.modify_symbol(stub_symbol);

        let prev_realm = self.current_realm.clone();
        let prev_slot_counter = self.spell_slot_counter;

        self.spell_slot_counter = 0;

        self.current_realm = Realm::Spell;
        self.spell_stack.push(spell_name.clone());

        // analyze the body of the spell
        let woven_body = self.analyze_statement(*body)?;

        self.spell_stack.pop();

        // Reset spell slot counter when exiting spell
        // self.spell_slot_counter = 0;

        self.spell_slot_counter = prev_slot_counter;

        self.current_realm = prev_realm;

        let captured_vals = std::mem::replace(&mut self.current_upvalues, upvals_saved);
        let Some(s) = self.symbol_table.resolve(&spell_name) else {
            return self.error(
                &format!("Could not find '{}' across the realms of eira!", spell_name),
                name,
            );
        };
        let _ = {
            let mut kind = s.kind.borrow_mut();

            let spell_info = match &mut *kind {
                SymbolKind::Spell(i) => i,
                _ => {
                    return self.error(&format!("The symbol '{}' is not a spell", s.name), name);
                }
            };
            spell_info.upvalues = captured_vals.clone();

            spell_info.clone()
        };

        self.symbol_table.end_scope();

        // Restore base_depth
        self.spell_base_depth = saved_spell_base_depth;

        // overwrite the spell with updated information
        let symbol = self
            .symbol_table
            .define_spell(
                spell_name.clone(),
                spell_weave.clone(),
                SpellInfo {
                    name: spell_name.clone(),
                    reagents: w_reagents.clone(),
                    release_weave: ret_weave,
                    upvalues: captured_vals,
                },
                slot,
                None,
                visibility,
            )
            .unwrap();

        Ok(WovenDecl::Spell {
            name: name,
            reagents: w_reagents,
            body: Box::new(woven_body),
            spell_symbol: symbol,
        })
    }
}
