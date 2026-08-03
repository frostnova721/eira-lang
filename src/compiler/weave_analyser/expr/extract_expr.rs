use crate::{Token, compiler::{Expr, WovenExpr, weaves::Weave}, weave_analyser::{WeaveAnalyzer, WeaveResult}};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_extract_expr(
        &mut self,
        _expected_weave: Option<&Weave>,
        deck: Expr,
        index: Expr,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
          let w_deck = self.analyze_expression(deck, None)?;
                let elem_weave = match w_deck.weave() {
                    Weave::Deck(weave, _) => *weave,
                    _ => {
                        self.error(
                            &format!(
                                "'{}' was expected to be a 'Deck' but its a '{}'!",
                                w_deck.token().lexeme,
                                w_deck.weave().get_name(),
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
                        "The index expression of a deck set operation must be of NumWeave!",
                        token.clone(),
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }

                Ok(WovenExpr::Extract {
                    deck: Box::new(w_deck),
                    index: Box::new(w_index),
                    weave: elem_weave,
                    token,
                })
    }
}