use crate::{
    Token,
    compiler::{Expr, Stmt, WovenStmt},
    weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_cycle_stmt(
        &mut self,
        variable: Token,
        iterable: Expr,
        body: Stmt,
    ) -> WeaveResult<WovenStmt> {
        Ok(WovenStmt::Cycle {
            variable,
            iterator: self.analyze_expression(iterable, None)?,
            body: Box::new(self.analyze_statement(body)?),
        })
    }
}
