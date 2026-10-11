use crate::{
    Token,
    compiler::{Expr, WovenExpr, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveError, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_vault_expr(
        &mut self,
        expected: Option<&Weave>,
        entries: Vec<(Expr, Expr)>,
        token: Token,
    ) -> WeaveResult<WovenExpr> {
        let (mut key_type, mut value_type, limit) = match expected {
            Some(Weave::Vault(k, v, n)) => (Some(*k.clone()), Some(*v.clone()), *n),
            None => (None, None, None),
            _ => {
                return Err(WeaveError {
                    msg: "Expected a Vault type for vault literal.".into(),
                    token,
                });
            }
        };
        let mut woven = Vec::with_capacity(entries.len());
        let mut known_keys = std::collections::HashSet::new();
        for (key, value) in entries {
            let k = self.analyze_expression(key, key_type.as_ref())?;
            let v = self.analyze_expression(value, value_type.as_ref())?;
            let kt = k.weave();
            let vt = v.weave();
            if !matches!(kt, Weave::Num | Weave::Text | Weave::Truth)
                || matches!(vt, Weave::Maybe(_) | Weave::Empty)
            {
                return Err(WeaveError {
                    msg: "Vault requires Num/Text/Truth keys and non-optional, non-empty values."
                        .into(),
                    token,
                });
            }
            if key_type.as_ref().is_some_and(|t| *t != kt)
                || value_type.as_ref().is_some_and(|t| *t != vt)
            {
                return Err(WeaveError {
                    msg: "Vault entries must match the declared key and value types.".into(),
                    token,
                });
            }
            if let WovenExpr::Literal { value, .. } = &k {
                if let Ok(key) = crate::values::vault::VaultKey::from_value(value) {
                    known_keys.insert(key);
                    if limit.is_some_and(|n| known_keys.len() > n) {
                        return Err(WeaveError {
                            msg: "Vault literal exceeds its maximum entry count.".into(),
                            token,
                        });
                    }
                }
            }
            key_type = Some(kt);
            value_type = Some(vt);
            woven.push((k, v));
        }
        let (Some(k), Some(v)) = (key_type, value_type) else {
            return Err(WeaveError {
                msg: "An empty vault needs a Vault<K, V> type annotation.".into(),
                token,
            });
        };
        Ok(WovenExpr::Vault {
            entries: woven,
            weave: Weave::Vault(Box::new(k), Box::new(v), limit),
            token,
        })
    }
}
