use crate::frontend::stmt::Stmt;

pub struct WeaveAnalyzer {
    ast: Vec<Stmt>,
}

impl WeaveAnalyzer {
    pub fn new(ast: Vec<Stmt>) -> Self {
        WeaveAnalyzer { ast: ast }
    }
    pub fn anaylze(&mut self) {
    }
}