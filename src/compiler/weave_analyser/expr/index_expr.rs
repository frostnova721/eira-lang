use crate::{
    Token,
    compiler::{Expr, WovenExpr, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_index_expr(
        &mut self,
        _expected_weave: Option<&Weave>,
        collection: Expr,
        index: Expr,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
        let w_collection = self.analyze_expression(collection, None)?;
        if let Weave::Vault(key, value, _) = w_collection.weave() {
            let w_index = self.analyze_expression(index, Some(&key))?;
            if w_index.weave() != *key {
                self.error("Vault lookup key has the wrong type.", token);
                return Ok(WovenExpr::Cursed { span: None });
            }
            return Ok(WovenExpr::Index {
                collection: Box::new(w_collection),
                index: Box::new(w_index),
                token,
                weave: Weave::Maybe(value),
            });
        }
        let elem_weave = match w_collection.weave() {
            Weave::Deck(weave, _) => *weave,
            _ => {
                self.error(
                    &format!(
                        "'{}' was expected to be a 'Deck' or 'Vault' but its a '{}'!",
                        w_collection.token().lexeme,
                        w_collection.weave().get_name(),
                    ),
                    token,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        };

        let w_index = self.analyze_expression(index, Some(&Weave::Num))?;

        let index_weave = w_index.weave();

        if index_weave != Weave::Num {
            self.error(
                "The index expression of a Deck lookup must be of NumWeave!",
                token.clone(),
            );
            return Ok(WovenExpr::Cursed { span: None });
        }

        Ok(WovenExpr::Index {
            collection: Box::new(w_collection),
            index: Box::new(w_index),
            weave: elem_weave,
            token,
        })
    }
}
