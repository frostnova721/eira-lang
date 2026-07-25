use crate::{
    Token,
    compiler::{Expr, WovenExpr, weaves::Weave},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_field_set_expr(
        &mut self,
        expected_weave: Option<&Weave>,
        material: Expr,
        property: Token,
        value: Expr,
    ) -> WeaveResult<WovenExpr> {
        let w_material_token = match self.analyze_expression(material, None)? {
            WovenExpr::Variable { name, .. } => name,
            _ => {
                self.error(
                    "Only variables can be accessed with '.' operator!",
                    property,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        };

        let Some(symbol) = self.symbol_table.resolve(&w_material_token.lexeme).cloned() else {
            self.error(
                &format!(
                    "The mark '{}' was not found across the eira realms!",
                    w_material_token.lexeme
                ),
                w_material_token,
            );
            return Ok(WovenExpr::Cursed { span: None });
        };

        let sign_name = match symbol.weave {
            Weave::Sign(ref name) => name,
            _ => {
                self.error(
                    "The mark 'n' is not a material of a sign!",
                    w_material_token,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
        };

        let Some(sign_symbol) = self.symbol_table.resolve(sign_name) else {
            self.error(
                &format!(
                    "The sign '{}' was not found across the eira realms!",
                    sign_name
                ),
                w_material_token,
            );
            return Ok(WovenExpr::Cursed { span: None });
        };

        let Some(sign_info) = sign_symbol.kind.borrow().get_sign_info() else {
            self.error(
                &format!("'{}' is not a sign!", sign_symbol.name),
                w_material_token,
            );
            return Ok(WovenExpr::Cursed { span: None });
        };

        let Some(mark) = sign_info.schema.get_field_index(property.lexeme.clone()) else {
            self.error(
                &format!(
                    "The mark '{}' is not defined for '{}'",
                    property.lexeme, sign_name
                ),
                property,
            );
            return Ok(WovenExpr::Cursed { span: None });
        };

        let Some(property_weave) = sign_info.marks.get(&property.lexeme) else {
            self.error(
                &format!(
                    "Eira couldn't find the weave for property '{}'",
                    property.lexeme
                ),
                w_material_token,
            );
            return Ok(WovenExpr::Cursed { span: None });
        };

        let w_material_expr = WovenExpr::Variable {
            name: w_material_token,
            weave: symbol.weave.clone(),
            symbol: symbol,
        };

        let w_value = self.analyze_expression(value, None)?;
        Ok(WovenExpr::FieldSet {
            material: Box::new(w_material_expr),
            property,
            value: Box::new(w_value),
            field_name_idx: mark as u16,
            weave: property_weave.clone(),
        })
    }
}
