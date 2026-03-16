use crate::frontend::{
    expr::{Expr, WovenExpr},
    stmt::{Stmt, WovenStmt},
    mark::{EtchedMark, WovenEtchedMark, Mark, WovenMark},
    reagents::{Reagent, WovenReagent},
};

const PIPE: &str = "│   ";
const BRANCH: &str = "├── ";
const LAST: &str = "└── ";
const EMPTY: &str = "    ";

pub struct AstPrinter {
    verbosity: u8,
    output: String,
}

impl AstPrinter {
    pub fn new(verbosity: u8) -> Self {
        AstPrinter {
            verbosity,
            output: String::new(),
        }
    }

    fn write(&mut self, prefix: &str, is_last: bool, text: &str) {
        let branch = if is_last { LAST } else { BRANCH };
        self.output.push_str(&format!("{}{}{}\n", prefix, branch, text));
    }

    fn next_prefix(prefix: &str, is_last: bool) -> String {
        format!("{}{}", prefix, if is_last { EMPTY } else { PIPE })
    }

    // ===== Parsed AST (Stmt/Expr) =====

    pub fn print_stmts(&mut self, stmts: &[Stmt]) -> String {
        self.output.clear();
        self.output.push_str("AST\n");
        let len = stmts.len();
        for (i, stmt) in stmts.iter().enumerate() {
            self.print_stmt("", stmt, i == len - 1);
        }
        self.output.clone()
    }

    fn print_stmt(&mut self, prefix: &str, stmt: &Stmt, is_last: bool) {
        match stmt {
            Stmt::ExprStmt { expr } => {
                self.write(prefix, is_last, "ExprStmt");
                self.print_expr(&Self::next_prefix(prefix, is_last), expr, true);
            }
            Stmt::VarDeclaration { name, mutable, initializer } => {
                let mut_str = if *mutable { "mut " } else { "" };
                self.write(prefix, is_last, &format!("VarDecl: {}{}", mut_str, name.lexeme));
                if let Some(init) = initializer {
                    self.print_expr(&Self::next_prefix(prefix, is_last), init, true);
                }
            }
            Stmt::Fate { condition, then_branch, else_branch } => {
                self.write(prefix, is_last, "Fate");
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, else_branch.is_none(), "condition:");
                self.print_expr(&Self::next_prefix(&next, else_branch.is_none()), condition, true);
                if let Some(else_b) = else_branch {
                    self.write(&next, false, "then:");
                    self.print_stmt(&Self::next_prefix(&next, false), then_branch, true);
                    self.write(&next, true, "else:");
                    self.print_stmt(&Self::next_prefix(&next, true), else_b, true);
                } else {
                    self.write(&next, true, "then:");
                    self.print_stmt(&Self::next_prefix(&next, true), then_branch, true);
                }
            }
            Stmt::While { condition, body } => {
                self.write(prefix, is_last, "While");
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, false, "condition:");
                self.print_expr(&Self::next_prefix(&next, false), condition, true);
                self.write(&next, true, "body:");
                self.print_stmt(&Self::next_prefix(&next, true), body, true);
            }
            Stmt::Chant { expression } => {
                self.write(prefix, is_last, "Chant");
                self.print_expr(&Self::next_prefix(prefix, is_last), expression, true);
            }
            Stmt::Block { statements } => {
                self.write(prefix, is_last, "Block");
                let next = Self::next_prefix(prefix, is_last);
                let len = statements.len();
                for (i, s) in statements.iter().enumerate() {
                    self.print_stmt(&next, s, i == len - 1);
                }
            }
            Stmt::Sever { token: _ } => {
                self.write(prefix, is_last, "Sever");
            }
            Stmt::Flow { token: _ } => {
                self.write(prefix, is_last, "Flow");
            }
            Stmt::Spell { name, reagents, body, return_weave } => {
                let ret_str = if let Some(rw) = return_weave {
                    format!(" -> {}", rw.base.lexeme)
                } else {
                    String::new()
                };
                self.write(prefix, is_last, &format!("Spell: {}{}", name.lexeme, ret_str));
                let next = Self::next_prefix(prefix, is_last);
                if !reagents.is_empty() {
                    self.write(&next, false, "reagents:");
                    let reagent_prefix = Self::next_prefix(&next, false);
                    let len = reagents.len();
                    for (i, r) in reagents.iter().enumerate() {
                        self.print_reagent(&reagent_prefix, r, i == len - 1);
                    }
                }
                self.write(&next, true, "body:");
                self.print_stmt(&Self::next_prefix(&next, true), body, true);
            }
            Stmt::Release { token: _, expr } => {
                self.write(prefix, is_last, "Release");
                if let Some(e) = expr {
                    self.print_expr(&Self::next_prefix(prefix, is_last), e, true);
                }
            }
            Stmt::Sign { name, marks } => {
                self.write(prefix, is_last, &format!("Sign: {}", name.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                let len = marks.len();
                for (i, m) in marks.iter().enumerate() {
                    self.print_mark(&next, m, i == len - 1);
                }
            }
        }
    }

    fn print_expr(&mut self, prefix: &str, expr: &Expr, is_last: bool) {
        match expr {
            Expr::Binary { left, right, operator } => {
                self.write(prefix, is_last, &format!("Binary: {}", operator.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                self.print_expr(&next, left, false);
                self.print_expr(&next, right, true);
            }
            Expr::Unary { operand, operator } => {
                self.write(prefix, is_last, &format!("Unary: {}", operator.lexeme));
                self.print_expr(&Self::next_prefix(prefix, is_last), operand, true);
            }
            Expr::Literal { value, token: _ } => {
                self.write(prefix, is_last, &format!("Literal: {:?}", value));
            }
            Expr::Variable { name } => {
                self.write(prefix, is_last, &format!("Variable: {}", name.lexeme));
            }
            Expr::Grouping { expression } => {
                self.write(prefix, is_last, "Grouping");
                self.print_expr(&Self::next_prefix(prefix, is_last), expression, true);
            }
            Expr::Assignment { name, value } => {
                self.write(prefix, is_last, &format!("Assign: {}", name.lexeme));
                self.print_expr(&Self::next_prefix(prefix, is_last), value, true);
            }
            Expr::Cast { reagents, callee } => {
                self.write(prefix, is_last, &format!("Cast: {}", callee.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                let len = reagents.len();
                for (i, r) in reagents.iter().enumerate() {
                    self.print_expr(&next, r, i == len - 1);
                }
            }
            Expr::Draw { marks, callee } => {
                self.write(prefix, is_last, &format!("Draw: {}", callee.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                let len = marks.len();
                for (i, m) in marks.iter().enumerate() {
                    self.print_etched_mark(&next, m, i == len - 1);
                }
            }
            Expr::Access { material, property } => {
                self.write(prefix, is_last, &format!("Access: .{}", property.lexeme));
                self.print_expr(&Self::next_prefix(prefix, is_last), material, true);
            }
        }
    }

    fn print_reagent(&mut self, prefix: &str, reagent: &Reagent, is_last: bool) {
        self.write(prefix, is_last, &format!("{}: {}", reagent.name.lexeme, reagent.weave_name.lexeme));
    }

    fn print_mark(&mut self, prefix: &str, mark: &Mark, is_last: bool) {
        self.write(prefix, is_last, &format!("{}: {}", mark.name.lexeme, mark.weave_name.lexeme));
    }

    fn print_etched_mark(&mut self, prefix: &str, mark: &EtchedMark, is_last: bool) {
        self.write(prefix, is_last, &format!("{} =", mark.name.lexeme));
        self.print_expr(&Self::next_prefix(prefix, is_last), &mark.expr, true);
    }

    // ===== Woven AST (WovenStmt/WovenExpr) =====

    pub fn print_woven_stmts(&mut self, stmts: &[WovenStmt]) -> String {
        self.output.clear();
        self.output.push_str("Woven AST\n");
        let len = stmts.len();
        for (i, stmt) in stmts.iter().enumerate() {
            self.print_woven_stmt("", stmt, i == len - 1);
        }
        self.output.clone()
    }

    fn print_woven_stmt(&mut self, prefix: &str, stmt: &WovenStmt, is_last: bool) {
        match stmt {
            WovenStmt::ExprStmt { expr } => {
                self.write(prefix, is_last, "ExprStmt");
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), expr, true);
            }
            WovenStmt::VarDeclaration { name, mutable, initializer, symbol } => {
                let mut_str = if *mutable { "mut " } else { "" };
                let sym_info = self.symbol_info(symbol);
                self.write(prefix, is_last, &format!("VarDecl: {}{}{}", mut_str, name.lexeme, sym_info));
                if let Some(init) = initializer {
                    self.print_woven_expr(&Self::next_prefix(prefix, is_last), init, true);
                }
            }
            WovenStmt::Fate { condition, then_branch, else_branch } => {
                self.write(prefix, is_last, "Fate");
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, else_branch.is_none(), "condition:");
                self.print_woven_expr(&Self::next_prefix(&next, else_branch.is_none()), condition, true);
                if let Some(else_b) = else_branch {
                    self.write(&next, false, "then:");
                    self.print_woven_stmt(&Self::next_prefix(&next, false), then_branch, true);
                    self.write(&next, true, "else:");
                    self.print_woven_stmt(&Self::next_prefix(&next, true), else_b, true);
                } else {
                    self.write(&next, true, "then:");
                    self.print_woven_stmt(&Self::next_prefix(&next, true), then_branch, true);
                }
            }
            WovenStmt::While { condition, body } => {
                self.write(prefix, is_last, "While");
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, false, "condition:");
                self.print_woven_expr(&Self::next_prefix(&next, false), condition, true);
                self.write(&next, true, "body:");
                self.print_woven_stmt(&Self::next_prefix(&next, true), body, true);
            }
            WovenStmt::Chant { expression } => {
                self.write(prefix, is_last, "Chant");
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), expression, true);
            }
            WovenStmt::Block { statements } => {
                self.write(prefix, is_last, "Block");
                let next = Self::next_prefix(prefix, is_last);
                let len = statements.len();
                for (i, s) in statements.iter().enumerate() {
                    self.print_woven_stmt(&next, s, i == len - 1);
                }
            }
            WovenStmt::Sever { token: _ } => {
                self.write(prefix, is_last, "Sever");
            }
            WovenStmt::Flow { token: _ } => {
                self.write(prefix, is_last, "Flow");
            }
            WovenStmt::Spell { name, reagents, body, spell } => {
                let ret_str = format!(" -> {:?}", spell.release_weave);
                self.write(prefix, is_last, &format!("Spell: {}{}", name.lexeme, ret_str));
                let next = Self::next_prefix(prefix, is_last);
                if !reagents.is_empty() {
                    self.write(&next, false, "reagents:");
                    let reagent_prefix = Self::next_prefix(&next, false);
                    let len = reagents.len();
                    for (i, r) in reagents.iter().enumerate() {
                        self.print_woven_reagent(&reagent_prefix, r, i == len - 1);
                    }
                }
                self.write(&next, true, "body:");
                self.print_woven_stmt(&Self::next_prefix(&next, true), body, true);
            }
            WovenStmt::Release { token: _, expr } => {
                self.write(prefix, is_last, "Release");
                if let Some(e) = expr {
                    self.print_woven_expr(&Self::next_prefix(prefix, is_last), e, true);
                }
            }
            WovenStmt::Sign { name, marks, info } => {
                let info_str = if self.verbosity >= 1 {
                    format!(" [slot:{}, fields:{}]", info.symbol.slot_idx, info.schema.field_count())
                } else {
                    String::new()
                };
                self.write(prefix, is_last, &format!("Sign: {}{}", name.lexeme, info_str));
                let next = Self::next_prefix(prefix, is_last);
                let len = marks.len();
                for (i, m) in marks.iter().enumerate() {
                    self.print_woven_mark(&next, m, i == len - 1);
                }
            }
        }
    }

    fn print_woven_expr(&mut self, prefix: &str, expr: &WovenExpr, is_last: bool) {
        match expr {
            WovenExpr::Binary { left, right, operator, tapestry } => {
                let tap = self.tapestry_info(tapestry);
                self.write(prefix, is_last, &format!("Binary: {}{}", operator.lexeme, tap));
                let next = Self::next_prefix(prefix, is_last);
                self.print_woven_expr(&next, left, false);
                self.print_woven_expr(&next, right, true);
            }
            WovenExpr::Unary { operand, operator, tapestry } => {
                let tap = self.tapestry_info(tapestry);
                self.write(prefix, is_last, &format!("Unary: {}{}", operator.lexeme, tap));
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), operand, true);
            }
            WovenExpr::Literal { value, token: _, tapestry } => {
                let tap = self.tapestry_info(tapestry);
                self.write(prefix, is_last, &format!("Literal: {:?}{}", value, tap));
            }
            WovenExpr::Variable { name, tapestry, symbol } => {
                let tap = self.tapestry_info(tapestry);
                let sym = self.symbol_info(symbol);
                self.write(prefix, is_last, &format!("Variable: {}{}{}", name.lexeme, sym, tap));
            }
            WovenExpr::Grouping { expression, tapestry } => {
                let tap = self.tapestry_info(tapestry);
                self.write(prefix, is_last, &format!("Grouping{}", tap));
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), expression, true);
            }
            WovenExpr::Assignment { name, value, tapestry, symbol } => {
                let tap = self.tapestry_info(tapestry);
                let sym = self.symbol_info(symbol);
                self.write(prefix, is_last, &format!("Assign: {}{}{}", name.lexeme, sym, tap));
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), value, true);
            }
            WovenExpr::Cast { reagents, callee, tapestry, spell_symbol } => {
                let tap = self.tapestry_info(tapestry);
                let sym = self.symbol_info(spell_symbol);
                self.write(prefix, is_last, &format!("Cast: {}{}{}", callee.lexeme, sym, tap));
                let next = Self::next_prefix(prefix, is_last);
                let len = reagents.len();
                for (i, r) in reagents.iter().enumerate() {
                    self.print_woven_expr(&next, r, i == len - 1);
                }
            }
            WovenExpr::Draw { marks, callee, tapestry, sign_info } => {
                let tap = self.tapestry_info(tapestry);
                let info_str = if self.verbosity >= 1 {
                    format!(" [slot:{}, fields:{}]", sign_info.symbol.slot_idx, sign_info.schema.field_count())
                } else {
                    String::new()
                };
                self.write(prefix, is_last, &format!("Draw: {}{}{}", callee.lexeme, info_str, tap));
                let next = Self::next_prefix(prefix, is_last);
                let len = marks.len();
                for (i, m) in marks.iter().enumerate() {
                    self.print_woven_etched_mark(&next, m, i == len - 1);
                }
            }
            WovenExpr::Access { material, property, field_name_idx, tapestry } => {
                let tap = self.tapestry_info(tapestry);
                let idx_str = if self.verbosity >= 1 {
                    format!(" [idx:{}]", field_name_idx)
                } else {
                    String::new()
                };
                self.write(prefix, is_last, &format!("Access: .{}{}{}", property.lexeme, idx_str, tap));
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), material, true);
            }
        }
    }

    fn print_woven_reagent(&mut self, prefix: &str, reagent: &WovenReagent, is_last: bool) {
        self.write(prefix, is_last, &format!("{}: {:?}", reagent.name.lexeme, reagent.weave));
    }

    fn print_woven_mark(&mut self, prefix: &str, mark: &WovenMark, is_last: bool) {
        self.write(prefix, is_last, &format!("{}: {:?}", mark.name.lexeme, mark.weave));
    }

    fn print_woven_etched_mark(&mut self, prefix: &str, mark: &WovenEtchedMark, is_last: bool) {
        self.write(prefix, is_last, &format!("{} =", mark.name.lexeme));
        self.print_woven_expr(&Self::next_prefix(prefix, is_last), &mark.expr, true);
    }

    fn symbol_info(&self, symbol: &crate::frontend::symbol_table::Symbol) -> String {
        if self.verbosity >= 1 {
            format!(" [slot:{}, depth:{}]", symbol.slot_idx, symbol.depth)
        } else {
            String::new()
        }
    }

    fn tapestry_info(&self, tapestry: &crate::frontend::tapestry::Tapestry) -> String {
        if self.verbosity >= 2 {
            format!(" <tap:0x{:X}>", tapestry.0)
        } else {
            String::new()
        }
    }
}

// Convenience functions
pub fn print_ast(stmts: &[Stmt], verbosity: u8) {
    if verbosity >= 3 {
        println!("{:#?}", stmts);
    } else {
        let mut printer = AstPrinter::new(verbosity);
        println!("{}", printer.print_stmts(stmts));
    }
}

pub fn print_woven_ast(stmts: &[WovenStmt], verbosity: u8) {
    if verbosity >= 3 {
        println!("{:#?}", stmts);
    } else {
        let mut printer = AstPrinter::new(verbosity);
        println!("{}", printer.print_woven_stmts(stmts));
    }
}
