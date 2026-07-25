use std::{cell::RefCell, collections::HashMap};

use crate::{
    Token,
    compiler::{
        ast::decl::WovenDecl,
        mark::{Mark, WovenMark},
        symbol_table::{Symbol, SymbolKind},
        types::Visibility,
        weaves::Weave,
    },
    values::sign::{SignInfo, SignSchema},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_sign(
        &mut self,
        name: Token,
        marks: Vec<Mark>,
        visibility: Option<Visibility>,
    ) -> WeaveResult<WovenDecl> {
        if let Some(_) = self.symbol_table.resolve_in_current_scope(&name.lexeme) {
            self.error(
                "A variable has been declared with same name as the sign.",
                name,
            );
            return Ok(WovenDecl::Cursed { span: None });
        }

        let visibility = visibility.unwrap_or(Visibility::default());

        let mut sign_info = SignInfo {
            schema: SignSchema::new(name.lexeme.clone()),
            marks: HashMap::new(),
            attunements: HashMap::new(),
        };

        let symbol = self.symbol_table.define_sign(
            name.lexeme.clone(),
            Weave::Sign(name.lexeme.clone()),
            sign_info.clone(),
            None,
            self.symbol_table.get_current_scope_size(),
            visibility.clone(),
        );

        let symbol = match symbol {
            Some(s) => s,
            _ => {
                // this shouldnt be thrown
                self.error(
                    "The sign could not be defined. [This should be thrown]",
                    name,
                );
                return Ok(WovenDecl::Cursed { span: None });
            }
        };

        let mut names: Vec<String> = vec![];
        let mut w_marks: Vec<WovenMark> = vec![];

        for m in marks {
            if names.contains(&m.name.lexeme) {
                self.error(
                    "A different mark with same name exists in the sign!",
                    m.name,
                );
                return Ok(WovenDecl::Cursed { span: None });
            }
            names.push(m.name.lexeme.clone());
            let mark_weave = self.analyze_parsed_weave(m.parsed_weave)?;
            w_marks.push(WovenMark {
                name: m.name.clone(),
                weave: mark_weave.clone(),
            });

            sign_info.marks.insert(m.name.lexeme.clone(), mark_weave);
            sign_info.schema.add_field(m.name.lexeme);
        }

        let new_symbol = Symbol {
            name: symbol.name,
            weave: symbol.weave,
            depth: symbol.depth,
            kind: RefCell::new(SymbolKind::Sign(sign_info)),
            slot_idx: symbol.slot_idx,
            parent: None,
            visibility: visibility,
        };

        self.symbol_table.modify_symbol(new_symbol.clone());

        Ok(WovenDecl::Sign {
            name,
            marks: w_marks,
            sign_symbol: new_symbol,
            // schema
        })
    }
}
