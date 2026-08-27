use crate::{
    Token, compiler::{Expr, Stmt, WovenStmt, types::Visibility, weaves::Weave}, weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_cycle_stmt(
        &mut self,
        variable: Token,
        iterable: Expr,
        body: Stmt,
    ) -> WeaveResult<WovenStmt> {
        self.symbol_table.new_scope();

        let iterable = self.analyze_expression(iterable, None)?;

        let weave_of_variable = match iterable.weave() {
            Weave::Deck(weave, _) => *weave,
            Weave::Range => Weave::Num,
            _ => {
                self.error(
                    &format!(
                        "Cannot iterate over a value of type {}",
                        iterable.weave().get_name()
                    ),
                    variable,
                );

                return Ok(WovenStmt::Cursed { span: None });
            }
        };

        let var_symbol = self.symbol_table.define_variable(
            variable.lexeme.clone(),
            weave_of_variable,
            true,
            self.symbol_table.get_current_scope_size(),
            None,
            Visibility::Secret,
        );

        let w_body = self.analyze_statement(body)?;

        self.symbol_table.end_scope();
        
        Ok(WovenStmt::Cycle {
            variable: var_symbol.unwrap(),
            iterable,
            body: Box::new(w_body),
        })
    }
}
