use crate::frontend::{expr::{Expr, WovenExpr}, scanner::Token, stmt::Stmt, tapestry::Tapestry};

struct WeaveError(String);

type WeaveResult<T> = Result<T, WeaveError>;
pub struct WeaveAnalyzer {
    ast: Vec<Stmt>,
}

impl WeaveAnalyzer {
    pub fn new(ast: Vec<Stmt>) -> Self {
        WeaveAnalyzer { ast: ast }
    }
    pub fn anaylze(&mut self) {
        self.analyze_statements(self.ast.clone());
    }

    fn analyze_statements(&self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            match stmt {
                Stmt::Block { statements } => {
                    self.analyze_statements(statements);
                }
                Stmt::Chant { expression } => {}
                Stmt::ExprStmt { expr } => {}
                Stmt::Fate {
                    condition,
                    then_branch,
                    else_branch,
                } => {}
                Stmt::VarDeclaration {
                    name,
                    mutable,
                    initializer,
                } => {}
                Stmt::While { condition, body } => {}
                Stmt::Sever => {}
            }
        }
    }

    fn analyze_expression(&self, expr: Expr) -> WeaveResult<WovenExpr> {
        match expr {
            Expr::Assignment { name, value } => {},
            Expr::Binary { left, right, operator } => {
                Ok(WovenExpr::Binary { left: left, right: right, operator: operator, tapestry: Tapestry::new(0) })
            },
            Expr::Grouping { expression } => {},
            Expr::Literal { value } => {},
            Expr::Unary { operand, operator } => {},
            Expr::Variable { name } => {},

        }
    }
}
