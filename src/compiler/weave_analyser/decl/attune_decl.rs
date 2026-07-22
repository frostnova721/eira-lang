use crate::{
    Token, compiler::{Stmt, WovenStmt, ast::decl::WovenDecl, symbol_table::SymbolKind}, weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl<'a> WeaveAnalyzer<'a> {
    pub(crate) fn analyze_attune(&mut self, sign: Token, spells: Vec<Box<Stmt>>) -> WeaveResult<WovenDecl> {
        // verify that the symbol exists and it is a sign
        let Some(sign_symbol) = self.symbol_table.resolve(&sign.lexeme) else {
            return self.error(
                &format!(
                    "No sign found across the eira realms with the name '{}'",
                    sign.lexeme
                ),
                sign,
            );
        };

        match *sign_symbol.kind.borrow() {
            SymbolKind::Sign(_) => {}
            _ => {
                return self.error(
                    &format!("The symbol '{}' is not a sign.", sign.lexeme),
                    sign,
                );
            }
        };

        // new scope, we dont want stuff colliding
        // self.symbol_table.new_scope();

        let mut w_spells: Vec<Box<WovenStmt>> = vec![];

        for spell in spells {
            let w_spell = self.analyze_statement(*spell)?;
            w_spells.push(Box::new(w_spell));
        }

        // self.symbol_table.end_scope();

        Ok(WovenDecl::Attune {
            sign: sign,
            spells: w_spells,
        })
    }
}
