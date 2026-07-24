use std::rc::Rc;

use crate::{
    Token,
    compiler::{
        Expr, ast::decl::WovenDecl, parser::types::ParsedWeave, symbol_table::Symbol,
        types::Visibility, weaves::Weave,
    },
    weave_analyser::{Realm, WeaveAnalyzer, WeaveError, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_var_decl(
        &mut self,
        name: Token,
        mutable: bool,
        initializer: Option<Expr>,
        weave: Option<ParsedWeave>,
        visibility: Option<Visibility>,
    ) -> WeaveResult<WovenDecl> {
        // allow variable shadowing from outer scopes
        if let Some(_symbol) = self.symbol_table.resolve_in_current_scope(&name.lexeme) {
            self.error(
                &format!(
                    "The variable '{}' already exists in the current scope!",
                    name.lexeme
                ),
                name,
            );

            return Ok(WovenDecl::Cursed { span: None });
        }

        let visibility = visibility.unwrap_or(Visibility::default());

        let expr_weave: Result<Weave, WeaveError>;
        let mut specified_weave: Option<Weave> = None;

        if weave.is_some() {
            specified_weave = Some(self.analyze_parsed_weave(weave.clone().unwrap())?);
        }

        let w_initializer = match initializer {
            Some(val) => {
                let w = if specified_weave.is_some() {
                    Some(specified_weave.as_ref().unwrap())
                } else {
                    None
                };
                Some(self.analyze_expression(val, w)?)
            }
            _ => None,
        };

        // kinda double check the weave (this should be a redundant check, the analyze_expr should 
        // take care of the checks)
        if let Some(ref sw) = specified_weave && w_initializer.is_some() {
            let init_weave = w_initializer.as_ref().unwrap().weave();
            if *sw != init_weave {
                self.error(
                    &format!(
                        "The specified weave '{:?}' does not match the initializer's weave '{:?}'!",
                        sw, init_weave
                    ),
                    name,
                );
                return Ok(WovenDecl::Cursed { span: None });
            }
            
        }

        let mut parent: Option<Rc<Symbol>> = None;

        match &w_initializer {
            Some(val) => {
                // Try to get weave from symbol first (for composite weaves like SpellWeave<TextWeave>)
                expr_weave = if let Some(symbol) = val.symbol() {
                    parent = Some(Rc::new(symbol.clone()));
                    Ok(val.weave())
                } else {
                    Ok(val.weave())
                };
            }
            None => {
                if !mutable {
                    // this shouldnt occur since parser should already have handled this
                    self.error("bind values must be initialized with an expression!", name);
                    return Ok(WovenDecl::Cursed { span: None });
                }

                // if no initializer, the weave must be specified. Try to get weave from the specified weave name
                expr_weave = match specified_weave {
                    Some(ref s_w) => Ok(s_w.clone()),
                    None => {
                        self.error("Couldn't determine a weave for the variable! You shall specify a weave for uninitialized variables!", name.clone());
                        return Ok(WovenDecl::Cursed { span: None });
                    }
                }
            }
        }

        let slot = if matches!(self.current_realm, Realm::Spell) {
            // Inside a spell, use continuous slot counter
            let current_slot = self.spell_slot_counter;
            self.spell_slot_counter += 1;
            current_slot
        } else {
            // Outside spells, use scope-local slot assignment
            self.symbol_table.get_current_scope_size()
        };

        // use the explicit weave if defined/available
        let weave_for_symbol = if specified_weave.is_some() {
            specified_weave.unwrap()
        } else {
            expr_weave?
        };

        let s = self
            .symbol_table
            .define_variable(
                name.lexeme.clone(),
                weave_for_symbol,
                mutable,
                slot,
                parent,
                visibility,
            )
            .unwrap();

        Ok(WovenDecl::VarDeclaration {
            name: name,
            mutable: mutable,
            initializer: w_initializer,
            symbol: s,
        })
    }
}
