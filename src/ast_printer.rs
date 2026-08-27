use crate::compiler::{
    Expr, Stmt, WovenExpr, WovenStmt, ast::decl::{Decl, WovenDecl}, mark::{EtchedMark, Mark, WovenEtchedMark, WovenMark}, reagents::{Reagent, WovenReagent}, types::Visibility,
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
        self.output
            .push_str(&format!("{}{}{}\n", prefix, branch, text));
    }

    fn next_prefix(prefix: &str, is_last: bool) -> String {
        format!("{}{}", prefix, if is_last { EMPTY } else { PIPE })
    }

    // ===== Parsed AST (Stmt/Expr) =====

    pub fn print_decls(&mut self, decls: &[crate::compiler::ast::decl::Decl]) -> String {
        self.output.clear();
        self.output.push_str("AST\n");
        let len = decls.len();
        for (i, decl) in decls.iter().enumerate() {
            self.print_decl("", decl, i == len - 1);
        }
        self.output.clone()
    }

    fn print_decl(&mut self, prefix: &str, decl: &crate::compiler::ast::decl::Decl, is_last: bool) {
        match decl {
            Decl::Statement { stmt, token: _ } => {
                self.print_stmt(prefix, stmt, is_last)
            }
            Decl::VarDeclaration { name, mutable, initializer, weave, visibility } => {
                let mut_str = if *mutable { "mut " } else { "" };
                let vis_str = if *visibility == Some(Visibility::Public) {
                    "pub "
                } else {
                    ""
                };

                let weave_str = if let Some(w) = weave {
                    format!(": {}", w.base.lexeme)
                } else {
                    String::new()
                };

                self.write(
                    prefix,
                    is_last,
                    &format!(
                        "VarDeclaration: {}{}{}{}",
                        vis_str, mut_str, name.lexeme, weave_str
                    ),
                );
                if let Some(init) = initializer {
                    self.print_expr(&Self::next_prefix(prefix, is_last), init, true);
                }
            },
            Decl::Spell { name, reagents, body, return_weave, visibility, attuned_to } => {
                let ret_str = if let Some(rw) = return_weave {
                    format!(" -> {}", rw.base.lexeme)
                } else {
                    String::new()
                };

                let vis_str = if *visibility == Some(Visibility::Public) {
                    "pub "
                } else {
                    ""
                };
                let att_str = if let Some(att) = attuned_to {
                    format!(" attuned to {}", att.lexeme)
                } else {
                    String::new()
                };

                self.write(
                    prefix,
                    is_last,
                    &format!(
                        "Spell: {}{}{}{}",
                        vis_str, name.lexeme, ret_str, att_str
                    ),
                );

                let child_prefix = Self::next_prefix(prefix, is_last);
                let statements = if let Stmt::Block { statements } = &**body { statements.as_slice() } else { &[] };
                let total_children = reagents.len() + statements.len();
                let mut child_idx = 0;

                for reagent in reagents {
                    self.print_reagent(&child_prefix, reagent, child_idx == total_children - 1);
                    child_idx += 1;
                }
                for stmt in statements {
                    self.print_stmt(&child_prefix, stmt, child_idx == total_children - 1);
                    child_idx += 1;
                }
                
            },
            Decl::Sign { name, marks, visibility } => {
                let vis_str = if *visibility == Some(Visibility::Public) {
                    "pub "
                } else {
                    ""
                };
                self.write(prefix, is_last, &format!("Sign: {}{}", vis_str, name.lexeme));
                let child_prefix = Self::next_prefix(prefix, is_last);
                let len = marks.len();
                for (i, mark) in marks.iter().enumerate() {
                    self.print_mark(&child_prefix, mark, i == len - 1);
                }
            },
            Decl::Attune { sign, spells } => {
                self.write(prefix, is_last, &format!("Attune: {}", sign.lexeme));
                let child_prefix = Self::next_prefix(prefix, is_last);
                let len = spells.len();
                for (i, spell) in spells.iter().enumerate() {
                    self.print_stmt(&child_prefix, spell, i == len - 1);
                }
            },
            Decl::Tether { token: _, path, bind_to, is_path } => {
                let bind_str = if let Some(bt) = bind_to {
                    format!(" bind to {}", bt.lexeme)
                } else {
                    String::new()
                };

                let type_str = if *is_path { "Path" } else { "Package" };
                let path_str = if *is_path {
                    path[0].lexeme.clone()
                } else {
                    path.iter().map(|t| t.lexeme.clone()).collect::<Vec<_>>().join(".")
                };

                self.write(
                    prefix,
                    is_last,
                    &format!("Tether: {} ({}){}", path_str, type_str, bind_str),
                );
            },
            Decl::Cursed { span } => {
                self.write(prefix, is_last, &format!("Cursed: {:?}", span));
            },
        }
    }

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
            Stmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => {
                self.write(prefix, is_last, "Fate");
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, false, "condition:");
                self.print_expr(
                    &Self::next_prefix(&next, false),
                    condition,
                    true,
                );
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
            Stmt::Release { token: _, expr } => {
                self.write(prefix, is_last, "Release");
                if let Some(e) = expr {
                    self.print_expr(&Self::next_prefix(prefix, is_last), e, true);
                }
            }
            Stmt::Vanish { target, token } => {
                self.write(prefix, is_last, &format!("Vanish: {}", token.lexeme));
                self.print_expr(&Self::next_prefix(prefix, is_last), target, true);
            }
            Stmt::Declaration(decl) => {
                self.print_decl(prefix, decl, is_last);
            }
            Stmt::Cursed { span } => {
                self.write(prefix, is_last, &format!("Cursed: {:?}", span));
            },
            Stmt::Cycle { iterable, variable, body } => {
                self.write(prefix, is_last, &format!("Cycle: {}", variable.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, false, "iterable:");
                self.print_expr(&Self::next_prefix(&next, false), iterable, true);
                self.write(&next, true, "body:");
                self.print_stmt(&Self::next_prefix(&next, true), body, true);
            },
        }
    }

    fn print_expr(&mut self, prefix: &str, expr: &Expr, is_last: bool) {
        match expr {
            Expr::Binary {
                left,
                right,
                operator,
            } => {
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
            Expr::Cast {
                reagents,
                callee,
                token: _,
            } => {
                self.write(prefix, is_last, "Cast");
                self.print_expr(&Self::next_prefix(prefix, is_last), callee, reagents.is_empty());
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
            Expr::Deck { elements, token: _ } => {
                self.write(prefix, is_last, "Deck");
                let next = Self::next_prefix(prefix, is_last);
                if elements.is_empty() {
                    self.write(&next, true, "(empty)");
                } else {
                    let len = elements.len();
                    for (i, element) in elements.iter().enumerate() {
                        self.print_expr(&next, element, i == len - 1);
                    }
                }
            }
            Expr::Extract {
                deck,
                index,
                token: _,
            } => {
                self.write(prefix, is_last, "Extract");
                let next = Self::next_prefix(prefix, is_last);
                self.print_expr(&next, deck, false);
                self.print_expr(&next, index, true);
            }
            Expr::DeckSet {
                deck,
                index,
                value,
                token: _,
            } => {
                self.write(prefix, is_last, "DeckSet");
                let next = Self::next_prefix(prefix, is_last);
                self.print_expr(&next, deck, false);
                self.print_expr(&next, index, false);
                self.print_expr(&next, value, true);
            }
            Expr::FieldSet {
                material,
                property,
                value,
            } => {
                self.write(prefix, is_last, &format!("FieldSet: .{}", property.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                self.print_expr(&next, material, false);
                self.print_expr(&next, value, true);
            }
            Expr::Blank { token } => {
                self.write(prefix, is_last, &format!("Blank: {}", token.lexeme));
            }
            Expr::Manifests { value, token } => {
                self.write(prefix, is_last, &format!("Manifests: {}", token.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                self.print_expr(&next, value, true);
            }
            Expr::SafeAccess { material, property } => {
                self.write(
                    prefix,
                    is_last,
                    &format!("SafeAccess: .{}", property.lexeme),
                );
                self.print_expr(&Self::next_prefix(prefix, is_last), material, true);
            }
            Expr::AssertSafe { operand, operator } => {
                self.write(prefix, is_last, &format!("AssertSafe: {}", operator.lexeme));
                self.print_expr(&Self::next_prefix(prefix, is_last), operand, true);
            }
            Expr::Cursed { span } => {
                self.write(prefix, is_last, &format!("Cursed: {:?}", span));
            },
            Expr::Range { start, end, token } => {
                self.write(prefix, is_last, &format!("Range: {}", token.lexeme));
                let next = Self::next_prefix(prefix, is_last);
                self.print_expr(&next, start, false);
                self.print_expr(&next, end, true);
            },
        }
    }

    fn print_reagent(&mut self, prefix: &str, reagent: &Reagent, is_last: bool) {
        self.write(
            prefix,
            is_last,
            &format!("{}: {}", reagent.name.lexeme, reagent.weave.base.lexeme),
        );
    }

    fn print_mark(&mut self, prefix: &str, mark: &Mark, is_last: bool) {
        self.write(
            prefix,
            is_last,
            &format!("{}: {}", mark.name.lexeme, mark.parsed_weave.base.lexeme),
        );
    }

    fn print_etched_mark(&mut self, prefix: &str, mark: &EtchedMark, is_last: bool) {
        self.write(prefix, is_last, &format!("{} =", mark.name.lexeme));
        self.print_expr(&Self::next_prefix(prefix, is_last), &mark.expr, true);
    }

    // ===== Woven AST (WovenStmt/WovenExpr) =====

    pub fn print_woven_decls(&mut self, decls: &[crate::compiler::ast::decl::WovenDecl]) -> String {
        self.output.clear();
        self.output.push_str("Woven AST\n");
        let len = decls.len();
        for (i, decl) in decls.iter().enumerate() {
            self.print_woven_decl("", decl, i == len - 1);
        }
        self.output.clone()
    }

    fn print_woven_decl(
        &mut self,
        prefix: &str,
        decl: &crate::compiler::ast::decl::WovenDecl,
        is_last: bool,
    ) {
        match decl {
            
            WovenDecl::Statement { stmt, token: _ } => {
                self.print_woven_stmt(prefix, stmt, is_last)
            }
            WovenDecl::VarDeclaration { name: _, mutable: _, initializer, symbol } => {
                self.write(
                    prefix,
                    is_last,
                    &format!("VarDeclaration: {}", symbol.name),
                );
                if let Some(init) = initializer {
                    self.print_woven_expr(&Self::next_prefix(prefix, is_last), init, true);
                }
            },
            WovenDecl::Spell { name, reagents, body, spell_symbol: _spell_symbol } => {
                self.write(
                    prefix,
                    is_last,
                    &format!("Spell: {} -> ...", name.lexeme),
                );

                let child_prefix = Self::next_prefix(prefix, is_last);
                let statements = if let WovenStmt::Block { statements } = &**body { statements.as_slice() } else { &[] };
                let total_children = reagents.len() + statements.len();
                let mut child_idx = 0;

                for reagent in reagents {
                    self.print_woven_reagent(&child_prefix, reagent, child_idx == total_children - 1);
                    child_idx += 1;
                }
                for stmt in statements {
                    self.print_woven_stmt(&child_prefix, stmt, child_idx == total_children - 1);
                    child_idx += 1;
                }
            },
            WovenDecl::Sign { name, marks, sign_symbol: _ } => {
                self.write(prefix, is_last, &format!("Sign: {}", name.lexeme));
                let child_prefix = Self::next_prefix(prefix, is_last);
                let len = marks.len();
                for (i, mark) in marks.iter().enumerate() {
                    self.print_woven_mark(&child_prefix, mark, i == len - 1);
                }
            },
            WovenDecl::Attune { sign, spells } => {
                self.write(prefix, is_last, &format!("Attune: {}", sign.lexeme));
                let child_prefix = Self::next_prefix(prefix, is_last);
                let len = spells.len();
                for (i, spell) in spells.iter().enumerate() {
                    self.print_woven_stmt(&child_prefix, spell, i == len - 1);
                }
            },
            WovenDecl::Tether { statements, path, bind_to: _bind_to } => {
                self.write(prefix, is_last, &format!("Tether: {}", path));
                let child_prefix = Self::next_prefix(prefix, is_last);
                let len = statements.len();
                for (i, stmt) in statements.iter().enumerate() {
                    self.print_woven_decl(&child_prefix, stmt, i == len - 1);
                }
            },
            WovenDecl::Cursed { span } => {
                self.write(prefix, is_last, &format!("Cursed: {:?}", span));
            },
        }
    }

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
            WovenStmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => {
                self.write(prefix, is_last, "Fate");
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, false, "condition:");
                self.print_woven_expr(
                    &Self::next_prefix(&next, false),
                    condition,
                    true,
                );
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
            WovenStmt::Release { token: _, expr } => {
                self.write(prefix, is_last, "Release");
                if let Some(e) = expr {
                    self.print_woven_expr(&Self::next_prefix(prefix, is_last), e, true);
                }
            }
            WovenStmt::Declaration(decl) => {
                self.print_woven_decl(prefix, decl, is_last);
            }
            WovenStmt::Cursed { span } => {
                self.write(prefix, is_last, &format!("Cursed: {:?}", span));
            },
            WovenStmt::Cycle { iterable, variable, body } => {
                self.write(prefix, is_last, &format!("Cycle: {}", variable.name));
                let next = Self::next_prefix(prefix, is_last);
                self.write(&next, false, "iterable:");
                self.print_woven_expr(&Self::next_prefix(&next, false), iterable, true);
                self.write(&next, true, "body:");
                self.print_woven_stmt(&Self::next_prefix(&next, true), body, true);
            },
        }
    }

    fn print_woven_expr(&mut self, prefix: &str, expr: &WovenExpr, is_last: bool) {
        match expr {
            WovenExpr::Binary {
                left,
                right,
                operator,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(
                    prefix,
                    is_last,
                    &format!("Binary: {}{}", operator.lexeme, tap),
                );
                let next = Self::next_prefix(prefix, is_last);
                self.print_woven_expr(&next, left, false);
                self.print_woven_expr(&next, right, true);
            }
            WovenExpr::Unary {
                operand,
                operator,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(
                    prefix,
                    is_last,
                    &format!("Unary: {}{}", operator.lexeme, tap),
                );
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), operand, true);
            }
            WovenExpr::Literal {
                value,
                token: _,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(prefix, is_last, &format!("Literal: {:?}{}", value, tap));
            }
            WovenExpr::Variable {
                name,
                weave,
                symbol,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let sym = self.symbol_info(symbol);
                self.write(
                    prefix,
                    is_last,
                    &format!("Variable: {}{}{}", name.lexeme, sym, tap),
                );
            }
            WovenExpr::Grouping { expression, weave } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(prefix, is_last, &format!("Grouping{}", tap));
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), expression, true);
            }
            WovenExpr::Assignment {
                name,
                value,
                weave,
                symbol,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let sym = self.symbol_info(symbol);
                self.write(
                    prefix,
                    is_last,
                    &format!("Assign: {}{}{}", name.lexeme, sym, tap),
                );
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), value, true);
            }
            WovenExpr::Cast {
                reagents,
                callee,
                weave,
                spell_symbol,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let sym = self.symbol_info(spell_symbol);
                self.write(
                    prefix,
                    is_last,
                    &format!("Cast: {}{}{}", callee.lexeme, sym, tap),
                );
                let next = Self::next_prefix(prefix, is_last);
                let len = reagents.len();
                for (i, r) in reagents.iter().enumerate() {
                    self.print_woven_expr(&next, r, i == len - 1);
                }
            }
            WovenExpr::Draw {
                marks,
                callee,
                weave,
                sign_symbol,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let info_str = if self.verbosity >= 1 {
                    format!(
                        " [slot:{}, fields:{}]",
                        sign_symbol.slot_idx,
                        sign_symbol
                            .kind
                            .borrow()
                            .get_sign_info()
                            .unwrap()
                            .schema
                            .field_count()
                    )
                } else {
                    String::new()
                };
                self.write(
                    prefix,
                    is_last,
                    &format!("Draw: {}{}{}", callee.lexeme, info_str, tap),
                );
                let next = Self::next_prefix(prefix, is_last);
                let len = marks.len();
                for (i, m) in marks.iter().enumerate() {
                    self.print_woven_etched_mark(&next, m, i == len - 1);
                }
            }
            WovenExpr::Access {
                material,
                property,
                field_name_idx,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let idx_str = if self.verbosity >= 1 {
                    format!(" [idx:{}]", field_name_idx)
                } else {
                    String::new()
                };
                self.write(
                    prefix,
                    is_last,
                    &format!("Access: .{}{}{}", property.lexeme, idx_str, tap),
                );
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), material, true);
            }
            WovenExpr::Deck { elements, weave } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(prefix, is_last, &format!("Deck{}", tap));
                let next = Self::next_prefix(prefix, is_last);
                if elements.is_empty() {
                    self.write(&next, true, "(empty)");
                } else {
                    let len = elements.len();
                    for (i, element) in elements.iter().enumerate() {
                        self.print_woven_expr(&next, element, i == len - 1);
                    }
                }
            }
            WovenExpr::Extract {
                deck,
                index,
                token: _,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(prefix, is_last, &format!("Extract{}", tap));
                let next = Self::next_prefix(prefix, is_last);
                self.print_woven_expr(&next, deck, false);
                self.print_woven_expr(&next, index, true);
            }
            WovenExpr::DeckSet {
                deck,
                index,
                value,
                token: _,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(prefix, is_last, &format!("DeckSet{}", tap));
                let next = Self::next_prefix(prefix, is_last);
                self.print_woven_expr(&next, deck, false);
                self.print_woven_expr(&next, index, false);
                self.print_woven_expr(&next, value, true);
            }
            WovenExpr::FieldSet {
                material,
                property,
                value,
                field_name_idx,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let idx_str = if self.verbosity >= 1 {
                    format!(" [idx:{}]", field_name_idx)
                } else {
                    String::new()
                };
                self.write(
                    prefix,
                    is_last,
                    &format!("FieldSet: .{}{}{}", property.lexeme, idx_str, tap),
                );
                let next = Self::next_prefix(prefix, is_last);
                self.print_woven_expr(&next, material, false);
                self.print_woven_expr(&next, value, true);
            }
            WovenExpr::Manifests {
                value,
                token,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(
                    prefix,
                    is_last,
                    &format!("Manifests: {}{}", token.lexeme, tap),
                );
                let next = Self::next_prefix(prefix, is_last);
                self.print_woven_expr(&next, value, true);
            }
            WovenExpr::SafeAccess {
                material,
                property,
                field_name_idx,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let idx_str = if self.verbosity >= 1 {
                    format!(" [idx:{}]", field_name_idx)
                } else {
                    String::new()
                };
                self.write(
                    prefix,
                    is_last,
                    &format!("SafeAccess: .{}{}{}", property.lexeme, idx_str, tap),
                );
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), material, true);
            }
            WovenExpr::AssertSafe {
                operand,
                operator,
                weave,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(
                    prefix,
                    is_last,
                    &format!("AssertSafe: {}{}", operator.lexeme, tap),
                );
                self.print_woven_expr(&Self::next_prefix(prefix, is_last), operand, true);
            }
            WovenExpr::NativeCast {
                reagents,
                callee,
                weave,
                native_spell,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let sym = native_spell;
                self.write(
                    prefix,
                    is_last,
                    &format!("Cast: {}{:?}{}", callee.lexeme, sym, tap),
                );
                let next = Self::next_prefix(prefix, is_last);
                let len = reagents.len();
                for (i, r) in reagents.iter().enumerate() {
                    self.print_woven_expr(&next, r, i == len - 1);
                }
            }
            WovenExpr::SafeCast {
                reagents,
                callee,
                weave,
                spell_symbol,
            } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                let sym = spell_symbol;
                self.write(
                    prefix,
                    is_last,
                    &format!("SafeCast: {}{:?}{}", callee.lexeme, sym, tap),
                );
                let next = Self::next_prefix(prefix, is_last);
                let len = reagents.len();
                for (i, r) in reagents.iter().enumerate() {
                    self.print_woven_expr(&next, r, i == len - 1);
                }
            }
            WovenExpr::BoundSpell {
                is_safe: _,
                material: _,
                spell_symbol,
                token,
                weave: _,
            } => {
                self.write(prefix, is_last, &format!("SHOULD NOT BE REACHED: BoundSpell in WovenExpr during printing! Spell: {}, Token: {}", spell_symbol.name, token.lexeme));
            }
            WovenExpr::Cursed { span } => {
                self.write(prefix, is_last, &format!("Cursed: {:?}", span));
            },
            WovenExpr::Range { start, end, token, weave } => {
                let tap = self.tapestry_info(&weave.get_tapestry());
                self.write(prefix, is_last, &format!("Range: {}{}", token.lexeme, tap));
                let next = Self::next_prefix(prefix, is_last);
                self.print_woven_expr(&next, start, false);
                self.print_woven_expr(&next, end, true);
            },
        }
    }

    fn print_woven_reagent(&mut self, prefix: &str, reagent: &WovenReagent, is_last: bool) {
        self.write(prefix, is_last, &format!("{:?}", reagent.weave));
    }

    fn print_woven_mark(&mut self, prefix: &str, mark: &WovenMark, is_last: bool) {
        self.write(
            prefix,
            is_last,
            &format!("{}: {:?}", mark.name.lexeme, mark.weave),
        );
    }

    fn print_woven_etched_mark(&mut self, prefix: &str, mark: &WovenEtchedMark, is_last: bool) {
        self.write(prefix, is_last, &format!("{} =", mark.name.lexeme));
        self.print_woven_expr(&Self::next_prefix(prefix, is_last), &mark.expr, true);
    }

    fn symbol_info(&self, symbol: &crate::compiler::symbol_table::Symbol) -> String {
        if self.verbosity >= 1 {
            format!(" [slot:{}, depth:{}]", symbol.slot_idx, symbol.depth)
        } else {
            String::new()
        }
    }

    fn tapestry_info(&self, tapestry: &crate::compiler::tapestry::Tapestry) -> String {
        if self.verbosity >= 2 {
            format!(" <tap:0x{:X}>", tapestry.0)
        } else {
            String::new()
        }
    }
}

// Convenience functions
pub fn print_ast(stmts: &[crate::compiler::ast::decl::Decl], verbosity: u8) {
    if verbosity >= 3 {
        println!("{:#?}", stmts);
    } else {
        let mut printer = AstPrinter::new(verbosity);
        println!("{}", printer.print_decls(stmts));
    }
}

pub fn print_woven_ast(stmts: &[crate::compiler::ast::decl::WovenDecl], verbosity: u8) {
    if verbosity >= 3 {
        println!("{:#?}", stmts);
    } else {
        let mut printer = AstPrinter::new(verbosity);
        println!("{}", printer.print_woven_decls(stmts));
    }
}
