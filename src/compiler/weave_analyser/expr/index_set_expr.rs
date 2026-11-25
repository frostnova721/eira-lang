use crate::{
    Token,
    compiler::{Expr, WovenExpr, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_index_set_expr(
        &mut self,
        _expected_weave: Option<&Weave>,
        collection: Expr,
        index: Expr,
        value: Expr,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
        let w_collection = self.analyze_expression(collection, None)?;
        if w_collection.is_cursed() {
            self.analyze_expression(index, None)?;
            self.analyze_expression(value, None)?;
            return Ok(WovenExpr::Cursed { span: None });
        }
        if let Weave::Vault(key, element, _) = w_collection.weave() {
            let w_index = self.analyze_expression(index, Some(&key))?;
            let w_value = self.analyze_expression(value, Some(&element))?;
            if !key.accepts(&w_index.weave()) || !element.accepts(&w_value.weave()) {
                self.error("Vault assignment key or value has the wrong type.", token);
                return Ok(WovenExpr::Cursed { span: None });
            }
            return Ok(WovenExpr::IndexSet {
                collection: Box::new(w_collection),
                index: Box::new(w_index),
                value: Box::new(w_value),
                token,
                weave: *element,
            });
        }
        let w_index = self.analyze_expression(index, Some(&Weave::Num))?;
        let w_value = self.analyze_expression(value, None)?;

        let index_weave = w_index.weave();

        if !Weave::Num.accepts(&index_weave) {
            self.error(
                "The index expression of a collection set operation must be of NumWeave!",
                token.clone(),
            );
            return Ok(WovenExpr::Cursed { span: None });
        }

        Ok(WovenExpr::IndexSet {
            collection: Box::new(w_collection),
            index: Box::new(w_index),
            value: Box::new(w_value.clone()),
            weave: w_value.weave(),
            token,
        })
    }
}
