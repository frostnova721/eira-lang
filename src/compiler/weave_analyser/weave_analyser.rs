use std::path::PathBuf;

use crate::{
    Value::{self},
    compiler::{
        Expr, Stmt, WovenExpr, WovenStmt,
        ast::decl::{Decl, WovenDecl},
        diagnostics::{Augury, CompilationPhase, SourceLocation},
        parser::types::ParsedWeave,
        scanner::Token,
        strand::{
            ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND,
            DIVISIVE_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, ITERABLE_STRAND, MAYBE_STRAND,
            MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND,
        },
        symbol_table::{Symbol, SymbolTable},
        token_type::TokenType,
        weave_analyser::WeaveAnalyzerContext,
        weaves::{Weave, Weaver},
    },
    values::spell::UpValue,
};

#[derive(Debug, Clone)]
pub struct WeaveError {
    pub msg: String,
    pub token: Token,
}

impl WeaveError {
    pub fn new(msg: &str, token: Token) -> Self {
        WeaveError {
            msg: msg.to_owned(),
            token: token,
        }
    }
}

pub type WeaveResult<T> = Result<T, WeaveError>;

#[derive(PartialEq, Clone)]
pub enum Realm {
    Genesis, // script level scope
    Spell,   // spell level scope
}

pub struct WeaveAnalyzer<'a> {
    pub(super) context: &'a mut WeaveAnalyzerContext,
    pub(super) augury: &'a mut Augury,

    pub(super) symbol_table: SymbolTable,
    pub(super) loop_depth: usize,
    pub(super) current_realm: Realm, // track the realm (scope type) the analyzer is in!
    pub(super) spell_stack: Vec<String>, // track the current spell name

    pub(super) current_upvalues: Vec<UpValue>, // upvalue for currently resolving spell
    pub(super) spell_base_depth: usize, // depth where current spell body starts (parameters live here)
    pub(super) spell_slot_counter: usize, // continuous slot counter within current spell
}

impl<'a> WeaveAnalyzer<'a> {
    pub fn new(context: &'a mut WeaveAnalyzerContext, augury: &'a mut Augury) -> Self {
        let st = SymbolTable::new();

        WeaveAnalyzer {
            context,
            augury,
            symbol_table: st,
            loop_depth: 0,
            current_realm: Realm::Genesis,
            spell_stack: vec![],
            current_upvalues: vec![],
            spell_base_depth: 0,
            spell_slot_counter: 0,
        }
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub(super) fn error(&mut self, msg: &str, token: Token) {
        let source = SourceLocation {
            file: PathBuf::from(self.context.source_path.clone()),
            line: token.line,
            column: token.column,
        };
        self.augury
            .forsee_curse(source, msg.to_string(), CompilationPhase::Weave);
        // Err(WeaveError::new(msg, token))
    }

    #[allow(dead_code)]
    pub(super) fn warn(&mut self, msg: &str, token: Token) {
        let source = SourceLocation {
            file: PathBuf::from(self.context.source_path.clone()),
            line: token.line,
            column: token.column,
        };
        self.augury
            .forsee_omen(source, msg.to_string(), CompilationPhase::Weave);
    }

    pub fn analyze(&mut self, ast: Vec<Decl>) -> WeaveResult<Vec<WovenDecl>> {
        self.analyze_decls(ast)
    }

    pub(super) fn analyze_decls(&mut self, decls: Vec<Decl>) -> WeaveResult<Vec<WovenDecl>> {
        let mut w_decls: Vec<WovenDecl> = Vec::new();
        for decl in decls {
            let woven = self.analyze_decl(decl)?;
            w_decls.push(woven);
        }
        Ok(w_decls)
    }

    pub(super) fn analyze_decl(&mut self, decl: Decl) -> WeaveResult<WovenDecl> {
        let is_tether_top_level = self.context.import_mode
            && self.symbol_table.get_depth() == 0
            && self.current_realm == Realm::Genesis;

        let woven = self.analyze_decl_inner(decl)?;

        match woven {
            WovenDecl::Attune { .. }
            | WovenDecl::Sign { .. }
            | WovenDecl::Spell { .. }
            | WovenDecl::VarDeclaration { .. }
            | WovenDecl::Tether { .. } => {}
            WovenDecl::Statement { ref token, stmt: _ } => {
                if is_tether_top_level {
                    self.error(
                        "Only declarations are allowed at top level in a tethered scroll.",
                        token.clone(),
                    );

                    return Ok(WovenDecl::Cursed { span: None });
                }
                // return self.error("Only declarations are allowed at top level.", token.clone());
            }
            WovenDecl::Cursed { .. } => {}
        }
        Ok(woven)
    }

    pub(super) fn analyze_statements(&mut self, stmts: Vec<Stmt>) -> WeaveResult<Vec<WovenStmt>> {
        let mut w_stmts: Vec<WovenStmt> = Vec::new();
        for stmt in stmts {
            let woven = self.analyze_statement(stmt)?;
            w_stmts.push(woven);
        }

        Ok(w_stmts)
    }

    pub(super) fn analyze_statement(&mut self, stmt: Stmt) -> WeaveResult<WovenStmt> {
        let woven = self.analyze_statement_inner(stmt)?;

        Ok(woven)
    }
    pub(super) fn analyze_decl_inner(&mut self, decl: Decl) -> WeaveResult<WovenDecl> {
        match decl {
            Decl::VarDeclaration {
                name,
                mutable,
                initializer,
                weave,
                visibility,
            } => self.analyze_var_decl(name, mutable, initializer, weave, visibility),
            Decl::Spell {
                name,
                reagents,
                body,
                return_weave,
                attuned_to,
                visibility,
            } => self.analyze_spell(name, reagents, visibility, return_weave, attuned_to, body),
            Decl::Sign {
                name,
                marks,
                visibility,
            } => self.analyze_sign(name, marks, visibility),
            Decl::Attune { sign, spells } => self.analyze_attune(sign, spells),

            Decl::Tether {
                token,
                path,
                bind_to,
                is_path,
            } => self.analyze_tether(token, path, bind_to, is_path),

            Decl::Statement { stmt, token } => Ok(WovenDecl::Statement {
                stmt: Box::new(self.analyze_statement(*stmt)?),
                token: token,
            }),

            Decl::Cursed { span: _ } => unreachable!(),
        }
    }

    fn analyze_statement_inner(&mut self, stmt: Stmt) -> WeaveResult<WovenStmt> {
        match stmt {
            Stmt::Declaration(decl) => {
                let w_decl = self.analyze_decl(*decl)?;
                Ok(WovenStmt::Declaration(Box::new(w_decl)))
            }
            Stmt::Block { statements } => {
                self.symbol_table.new_scope();
                let w_block = self.analyze_statements(statements)?;
                self.symbol_table.end_scope();
                return Ok(WovenStmt::Block {
                    statements: w_block,
                });
            }
            Stmt::Chant { expression } => {
                let w_expr = self.analyze_expression(expression, None)?;
                Ok(WovenStmt::Chant { expression: w_expr })
            }
            Stmt::ExprStmt { expr } => {
                let w_expr = self.analyze_expression(expr, None)?;
                Ok(WovenStmt::ExprStmt { expr: w_expr })
            }
            Stmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => self.analyze_fate_stmt(condition, *then_branch, else_branch),

            Stmt::While { condition, body } => self.analyze_while_stmt(condition, *body),
            Stmt::Sever { token } => {
                if self.loop_depth == 0 {
                    self.error("'sever' cannot be used outside a loop circle!", token);
                    return Ok(WovenStmt::Cursed { span: None });
                }
                Ok(WovenStmt::Sever { token })
            }
            Stmt::Flow { token } => {
                if self.loop_depth == 0 {
                    self.error("'flow' cannot be used outside a loop circle!", token);
                    return Ok(WovenStmt::Cursed { span: None });
                }
                Ok(WovenStmt::Flow { token })
            }
            Stmt::Release { token, expr } => self.analyze_release_stmt(token, expr),

            Stmt::Vanish { target, token } => self.analyze_vanish_stmt(token, target),
            Stmt::Cycle {
                iterable,
                variable,
                body,
            } => self.analyze_cycle_stmt(variable, iterable, *body),
            Stmt::Cursed { .. } => unreachable!(),
        }
    }

    pub(crate) fn analyze_expression(
        &mut self,
        expr: Expr,
        expected_weave: Option<&Weave>,
    ) -> WeaveResult<WovenExpr> {
        match expr {
            Expr::Binary {
                left,
                right,
                operator,
            } => self.analyze_binary_expr(expected_weave, *left, *right, operator),
            Expr::Grouping { expression } => self.analyze_expression(*expression, expected_weave),
            Expr::Literal { value, token } => {
                self.analyze_literal_expr(expected_weave, value, token)
            }

            Expr::Unary { operand, operator } => {
                self.analyze_unary_expr(expected_weave, *operand, operator)
            }
            Expr::Variable { name } => self.analyze_variable_expr(expected_weave, name),
            Expr::Assignment { name, value } => {
                self.analyze_assign_expr(expected_weave, name, *value)
            }
            Expr::Cast {
                reagents,
                callee,
                token,
            } => self.analyze_cast_expr(expected_weave, reagents, *callee, token),
            Expr::Draw { marks, callee } => self.analyze_draw_expr(expected_weave, marks, callee),
            Expr::Access { .. } | Expr::SafeAccess { .. } => {
                self.analyze_access_expr(expected_weave, expr)
            }
            Expr::Deck { elements, token } => {
                self.analyze_deck_expr(expected_weave, elements, token)
            }
            Expr::Extract { deck, index, token } => {
                self.analyze_extract_expr(expected_weave, *deck, *index, token)
            }
            Expr::DeckSet {
                deck,
                index,
                value,
                token,
            } => self.analyze_deck_set_expr(expected_weave, *deck, *index, *value, token),
            Expr::FieldSet {
                material,
                property,
                value,
            } => self.analyze_field_set_expr(expected_weave, *material, property, *value),
            Expr::Blank { token } => {
                self.error(
                    "Invalid '_' usage. '_' is used to assign a Empty value to Maybe<W> weaves!",
                    token,
                );
                return Ok(WovenExpr::Cursed { span: None });
            }
            Expr::Manifests { value, token } => {
                let w_value = self.analyze_expression(*value, None)?;

                // if !matches!(w_value.weave(), Weave::Maybe(_)) {
                //     return self.error(
                //         "The weave of the manifest expression must be a Maybe weave.",
                //         token,
                //     );
                // }

                Ok(WovenExpr::Manifests {
                    value: Box::new(w_value),
                    token,
                    weave: Weave::Truth,
                })
            }
            Expr::AssertSafe { operand, operator } => {
                let w_operand = self.analyze_expression(*operand, None)?;

                let weave = match w_operand.weave() {
                    Weave::Maybe(inner) => *inner,
                    _ => {
                        self.error(
                            "Safe Assertion can only be performed on Maybe<W> weaves!",
                            operator,
                        );
                        return Ok(WovenExpr::Cursed { span: None });
                    }
                };

                Ok(WovenExpr::AssertSafe {
                    operand: Box::new(w_operand),
                    operator,
                    weave: weave,
                })
            }
            Expr::Range { start, end, token } => {
                let w_start = self.analyze_expression(*start, Some(&Weave::Num))?;
                let w_end = self.analyze_expression(*end, Some(&Weave::Num))?;

                // Just to make sure...
                if matches!(w_start.weave(), Weave::Num) && matches!(w_end.weave(), Weave::Num) {
                    // valid range
                } else {
                    self.error(
                        "Range bounds must be of Num Weave!",
                        token.clone(),
                    );
                    return Ok(WovenExpr::Cursed { span: None });
                }

                Ok(WovenExpr::Range {
                    start: Box::new(w_start),
                    end: Box::new(w_end),
                    weave: Weave::Range,
                    token,
                })
            }
            Expr::Cursed { .. } => todo!(),
        }
    }

    pub(super) fn analyze_parsed_weave(&mut self, parsed_weave: ParsedWeave) -> WeaveResult<Weave> {
        let Some(base_weave) = self.get_weave_from_name(&parsed_weave.base.lexeme) else {
            self.error(
                &format!(
                    "Couldn't find {} weave across the realms of eira!",
                    parsed_weave.base.lexeme
                ),
                parsed_weave.base.clone(),
            );
            return Err(WeaveError {
                msg: "Couldn't find {} weave across the realms of eira!".to_owned(),
                token: parsed_weave.base,
            });
        };

        if !base_weave.can_sub_weave() && parsed_weave.inner.is_some() {
            self.error(
                &format!(
                    "{} weave cannot contain sub weaves!",
                    parsed_weave.base.lexeme
                ),
                parsed_weave.base.clone(),
            );
            return Err(WeaveError {
                msg: format!(
                    "{} weave cannot contain sub weaves!",
                    parsed_weave.base.lexeme
                )
                .to_owned(),
                token: parsed_weave.base,
            });
        }

        // at this point, its sure that only the sub weave-able weaves are processed
        let Some(inner_parsed_weave) = parsed_weave.inner else {
            return Ok(base_weave);
        };

        let inner_weave = self.analyze_parsed_weave(*inner_parsed_weave.clone())?;

        let weave = match base_weave {
            Weave::Deck(..) => {
                let res =
                    Weaver::weave_deck(base_weave, inner_weave.clone(), parsed_weave.capacity);
                if res.is_err() {
                    self.error(
                        &format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme,
                            inner_weave.get_name()
                        ),
                        inner_parsed_weave.base.clone(),
                    );
                    return Err(WeaveError {
                        msg: format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme,
                            inner_weave.get_name()
                        )
                        .to_owned(),
                        token: inner_parsed_weave.base,
                    });
                }
                res.unwrap()
            }
            Weave::Spell { .. } => {
                let res = Weaver::weave_spell(base_weave, inner_weave);
                if res.is_err() {
                    self.error(
                        &format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme, inner_parsed_weave.base.lexeme
                        ),
                        inner_parsed_weave.base.clone(),
                    );
                    return Err(WeaveError {
                        msg: format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme, inner_parsed_weave.base.lexeme
                        )
                        .to_owned(),
                        token: inner_parsed_weave.base,
                    });
                }
                res.unwrap()
            }
            Weave::Maybe(_) => {
                let res = Weaver::weave_maybe(base_weave, inner_weave);
                if res.is_err() {
                    self.error(
                        &format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme, inner_parsed_weave.base.lexeme
                        ),
                        inner_parsed_weave.base.clone(),
                    );
                    return Err(WeaveError {
                        msg: format!(
                            "Couldnt weave {} to {}",
                            parsed_weave.base.lexeme, inner_parsed_weave.base.lexeme
                        )
                        .to_owned(),
                        token: inner_parsed_weave.base,
                    });
                }
                res.unwrap()
            }
            _ => {
                self.error(
                    &format!(
                        "{} weave cannot contain any sub weaves!",
                        parsed_weave.base.lexeme
                    ),
                    parsed_weave.base.clone(),
                );
                return Err(WeaveError {
                    msg: format!(
                        "{} weave cannot contain any sub weaves!",
                        parsed_weave.base.lexeme
                    )
                    .to_owned(),
                    token: parsed_weave.base,
                });
            }
        };

        Ok(weave)
    }

    pub(super) fn get_core_scroll(&self, name: &str) -> Option<&str> {
        match name {
            "math" => Some(include_str!("../../../core_scrolls/math.eira")),
            _ => None,
        }
    }

    pub(super) fn can_assign(&self, expected: &Weave, provided: &Weave) -> bool {
        if expected == provided {
            return true;
        }

        match (expected, provided) {
            (Weave::Maybe(ai), b) => {
                if *b == Weave::Empty {
                    return true;
                }
                ai.as_ref() == provided
            }
            // (_, Weave::Maybe(bi)) => self.match_weave(expected, bi),
            _ => false,
        }
    }

    /// Resolve and add an upvalue for a symbol
    pub(super) fn resolve_n_add_upvalue(&mut self, symbol: &Symbol) -> WeaveResult<()> {
        // Only capture as upvalue if variable is from the spell's defining scope or outer
        // Parameters and locals have depth greater than the spell base depth
        if self.current_realm == Realm::Spell && symbol.depth <= self.spell_base_depth {
            // check if new. use both index and depth to avoid duplicates
            let slot = symbol.slot_idx;
            let is_new = !self
                .current_upvalues
                .iter()
                .any(|it| it.index == slot && it.depth == symbol.depth);

            if is_new {
                self.current_upvalues.push(UpValue {
                    index: slot,
                    closed: Value::Emptiness.into(),
                    depth: symbol.depth,
                });
            }
        }
        Ok(())
    }

    pub(super) fn strand_from_op(&self, op: TokenType) -> Option<u64> {
        match op {
            TokenType::Plus => Some(ADDITIVE_STRAND | CONCATINABLE_STRAND),
            TokenType::Minus => Some(SUBTRACTIVE_STRAND),
            TokenType::Star => Some(MULTIPLICATIVE_STRAND),
            TokenType::Slash => Some(DIVISIVE_STRAND),
            TokenType::Percent => Some(DIVISIVE_STRAND),
            TokenType::Bang => Some(CONDITIONAL_STRAND),
            TokenType::Greater
            | TokenType::Less
            | TokenType::GreaterEqual
            | TokenType::LessEqual => Some(ORDINAL_STRAND),
            TokenType::EqualEqual | TokenType::BangEqual => Some(EQUATABLE_STRAND),
            _ => None,
        }
    }

    /// Get the strand's name from its bit representation
    pub(super) fn strand_string_from_bits(&self, strand: u64) -> &str {
        match strand {
            ADDITIVE_STRAND => "ADDITIVE",
            SUBTRACTIVE_STRAND => "SUBTRACTIVE",
            MULTIPLICATIVE_STRAND => "MULTIPLICATIVE",
            DIVISIVE_STRAND => "DIVISIVE",
            ORDINAL_STRAND => "ORDINAL",
            CONDITIONAL_STRAND => "CONDITIONAL",
            CONCATINABLE_STRAND => "CONCATINABLE",
            INDEXIVE_STRAND => "INDEXIVE",
            ITERABLE_STRAND => "ITERABLE",
            EQUATABLE_STRAND => "EQUATABLE",
            CALLABLE_STRAND => "CALLABLE",
            MAYBE_STRAND => "MAYBE",
            NO_STRAND => "NONE",
            _ => "UNKNOWN",
        }
    }

    pub(super) fn get_weave_from_name(&mut self, name: &str) -> Option<Weave> {
        match name {
            "Num" => Some(Weave::Num),
            "Text" => Some(Weave::Text),
            "Truth" => Some(Weave::Truth),
            "Empty" => Some(Weave::Empty),
            "Sign" => Some(Weave::Sign(String::new())),
            "Spell" => Some(Weave::Spell {
                // reagents: vec![],
                release: Box::new(Weave::Empty),
            }),
            "Deck" => Some(Weave::Deck(Box::new(Weave::Empty), None)),
            "Maybe" => Some(Weave::Maybe(Box::new(Weave::Empty))),
            _ => {
                // match user defined types!
                let Some(symbol) = self.symbol_table.resolve(&name.to_string()) else {
                    return None;
                };

                if symbol.kind.borrow().get_sign_info().is_some() {
                    Some(Weave::Sign(name.to_owned()))
                } else {
                    None
                }
            }
        }
    }
}
