use crate::{Token, compiler::{WovenExpr, weaves::Weave}, weave_analyser::WeaveAnalyzer};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_variable_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        name: Token,
    ) -> crate::weave_analyser::WeaveResult<WovenExpr> {
        if let Some(symbol) = self.symbol_table.resolve(&name.lexeme).cloned() {
            //The symbol(variable) has been found
            self.resolve_n_add_upvalue(&symbol)?;

            let weave = &symbol.weave;

            if let Some(expected) = expected_weave {
                if *expected != *weave {
                    self.error(
                        &format!(
                            "The weave of the variable '{}' does not match the expected weave '{}'",
                            name.lexeme,
                            expected.get_name()
                        ),
                        name,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            }

            let woven = WovenExpr::Variable {
                name: name,
                weave: weave.clone(),
                symbol: symbol,
            };

            Ok(woven)
        } else {
            self.error(
                &format!("'{}' was undefined in the eira-verse!", name.lexeme),
                name,
            );
            return Ok(WovenExpr::Cursed { span: None });
        }
    }
}
