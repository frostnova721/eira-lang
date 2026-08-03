use crate::{
    Token,
    compiler::{Expr, WovenExpr, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_deck_set_expr(
        &mut self,
        _expected_weave: Option<&Weave>,
        deck: Expr,
        index: Expr,
        value: Expr,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
        let w_deck = self.analyze_expression(deck, None)?;
        let w_index = self.analyze_expression(index, Some(&Weave::Num))?;
        let w_value = self.analyze_expression(value, None)?;

        let index_weave = w_index.weave();

        if index_weave != Weave::Num {
            self.error(
                "The index expression of a deck set operation must be of NumWeave!",
                token.clone(),
            );
            return Ok(WovenExpr::Cursed { span: None });
        }

        Ok(WovenExpr::DeckSet {
            deck: Box::new(w_deck),
            index: Box::new(w_index),
            value: Box::new(w_value.clone()),
            weave: w_value.weave(),
            token,
        })
    }
}
