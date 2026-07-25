use crate::{
    Token,
    compiler::{WovenExpr, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
    values::Value,
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_literal_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        value: Value,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
        let weave = match value {
            Value::Number(_) => Weave::Num,
            Value::Emptiness => Weave::Empty,
            Value::Bool(_) => Weave::Truth,
            Value::String(_) => Weave::Text,
            _ => {
                self.error("Couldnt find a weave for the value", token.clone());
                return Ok(WovenExpr::Cursed { span: None });
            }
        };

        if let Some(expected) = expected_weave {
            if *expected != weave {
                self.error(
                    &format!(
                        "The weave of the literal value does not match the expected weave '{}'",
                        expected.get_name()
                    ),
                    token,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        }

        return Ok(WovenExpr::Literal {
            value: value,
            token: token,
            weave,
        });
    }
}
