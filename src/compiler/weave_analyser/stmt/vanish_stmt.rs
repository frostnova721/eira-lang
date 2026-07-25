use crate::{
    Token, Value, compiler::{Expr, WovenExpr, WovenStmt, symbol_table::SymbolKind, weaves::Weave}, weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_vanish_stmt(
        &mut self,
        token: Token,
        target: Expr,
    ) -> WeaveResult<WovenStmt> {
        let w_target = self.analyze_expression(target, None)?;

        match w_target.weave() {
            Weave::Maybe(_) => {}
            _ => {
                self.error(
                            "The weave of the target expression does not support vanishing (not a Maybe<W> weave).",
                            token,
                        );
                return Ok(WovenStmt::Cursed { span: None });
            }
        }

        match w_target.symbol() {
            Some(symbol) => match *symbol.kind.borrow() {
                SymbolKind::Variable { mutable } if !mutable => {
                    self.error("Cannot perform vanish for a bind-ed variable.", token);
                    return Ok(WovenStmt::Cursed { span: None });
                }
                _ => {}
            },
            None => {
                self.error("Cannot perform vanish on a non-variable expression.", token);
                return Ok(WovenStmt::Cursed { span: None });
            }
        }

        let empty_literal = WovenExpr::Literal {
            value: Value::Emptiness,
            token: token.clone(),
            weave: Weave::Empty,
        };

        // aka desugared
        let sugar_less = match w_target {
            WovenExpr::Access {
                material,
                property,
                field_name_idx,
                weave,
            } => WovenExpr::FieldSet {
                material,
                property,
                value: Box::new(empty_literal),
                field_name_idx,
                weave,
            },
            // WovenExpr::Assignment { name, value, weave, symbol } => {},
            WovenExpr::Variable {
                name,
                weave,
                symbol,
            } => WovenExpr::Assignment {
                name,
                value: Box::new(empty_literal),
                weave,
                symbol,
            },
            WovenExpr::Extract {
                deck,
                index,
                token,
                weave,
            } => WovenExpr::DeckSet {
                deck,
                index,
                value: Box::new(empty_literal),
                token,
                weave,
            },

            _ => {
                self.error("Cannot vanish from provided expression.", token);
                return Ok(WovenStmt::Cursed { span: None });
            }
        };

        return Ok(WovenStmt::ExprStmt { expr: sugar_less });
    }
}
