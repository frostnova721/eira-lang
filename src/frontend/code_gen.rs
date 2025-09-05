use crate::frontend::stmt::WovenStmt;

pub struct CodeGen {
    woven_ast: Vec<WovenStmt>,
}

impl CodeGen {

    pub fn new(w_ast: Vec<WovenStmt>) -> Self {
        CodeGen { woven_ast: w_ast }
    }

    // Thought this name is fun, nothing else, its the main entry point btw
    pub fn summon_bytecode(&mut self) {
        for stmt in self.woven_ast.iter() {
            
        }
    }

    fn gen_add_instruction() {}
}