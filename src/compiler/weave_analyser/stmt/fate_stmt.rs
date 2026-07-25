use crate::{
    compiler::{Expr, Stmt, WovenStmt, strand::CONDITIONAL_STRAND, }, weave_analyser::{WeaveAnalyzer, WeaveResult},
};

impl WeaveAnalyzer<'_> {
    pub(crate) fn analyze_fate_stmt(
        &mut self,
        condition: Expr,
        then_branch: Stmt,
        else_branch: Option<Box<Stmt>>,
    ) -> WeaveResult<WovenStmt> {
          let w_condition = self.analyze_expression(condition, None)?;

                if !w_condition
                    .weave()
                    .get_tapestry()
                    .has_strand(CONDITIONAL_STRAND)
                {
                    self.error(
                        "The condition provided to determine the fate does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    );
                    return Ok(WovenStmt::Cursed { span: None });
                }
                // scoping n stuff will be added by the block!
                let w_then = self.analyze_statement(then_branch)?;

                // self.symbol_table.end_scope();

                let w_else: Option<Box<WovenStmt>> = match else_branch {
                    Some(e_b) => Some(Box::new(self.analyze_statement(*e_b)?)),
                    None => None,
                };
                Ok(WovenStmt::Fate {
                    condition: w_condition,
                    then_branch: Box::new(w_then),
                    else_branch: w_else,
                })
    }
}