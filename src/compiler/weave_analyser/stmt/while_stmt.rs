use crate::{
    compiler::{Expr, Stmt, WovenStmt, strand::CONDITIONAL_STRAND},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_while_stmt(
        &mut self,
        condition: Expr,
        body: Stmt,
    ) -> WeaveResult<WovenStmt> {
        let w_condition = self.analyze_expression(condition, None)?;

        if !w_condition
            .weave()
            .get_tapestry()
            .has_strand(CONDITIONAL_STRAND)
        {
            self.error(
                        "The condition provided to determine the fate of loop does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    );
            return Ok(WovenStmt::Cursed { span: None });
        }

        // enter loop scope (for sever, flow purposes)
        self.loop_depth += 1;

        let w_body = self.analyze_statement(body)?;

        // loop scope exit
        self.loop_depth -= 1;

        Ok(WovenStmt::While {
            condition: w_condition,
            body: Box::new(w_body),
        })
    }
}
