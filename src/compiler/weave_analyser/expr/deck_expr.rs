use crate::{
    Token,
    compiler::{Expr, WovenExpr, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_deck_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        elements: Vec<Expr>,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
        let mut w_elements = vec![];

        let mut expected_capacity: Option<usize> = None;
        let mut prev_elem_weave: Option<Weave> = match expected_weave {
            Some(w) => match w {
                Weave::Deck(inner, c) => {
                    expected_capacity = *c;
                    Some(*inner.clone())
                }
                _ => {
                    self.error(
                        &format!(
                            "Hows this possible? a {} weave passed on to a deck!",
                            w.get_name()
                        ),
                        token,
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }
            },
            None => None,
        };

        if elements.len() > u8::MAX as usize {
            self.error("Deck size exceeds the maximum of 255 elements!", token);
            return Ok(WovenExpr::Cursed { span: None });
        }

        if let Some(c) = expected_capacity {
            if elements.len() > c {
                self.error(
                    &format!(
                        "The deck's specified capacity is {} while the length is {}",
                        c,
                        elements.len()
                    ),
                    token,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        }

        for element in &elements {
            let w_element = self.analyze_expression(element.clone(), None)?;
            let elem_weave = w_element.weave();
            if let Some(prev_weave) = prev_elem_weave {
                if elem_weave != prev_weave {
                    self.error("All elements of a deck must be of the same weave!", token);
                    return Ok(WovenExpr::Cursed { span: None });
                }
            }
            prev_elem_weave = Some(elem_weave);
            w_elements.push(w_element);
        }

        let weave = Weave::Deck(
            Box::new(prev_elem_weave.unwrap_or(Weave::Empty)),
            expected_capacity,
        );

        Ok(WovenExpr::Deck {
            elements: w_elements,
            weave: weave,
        })
    }
}
