use std::collections::HashMap;

use crate::{
    compiler::{
        Expr, Stmt, WovenExpr, WovenStmt,
        mark::{WovenEtchedMark, WovenMark},
        reagents::WovenReagent,
        scanner::Token,
        strand::{
            ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND,
            DIVISIVE_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, ITERABLE_STRAND,
            MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND,
        },
        symbol_table::{Symbol, SymbolTable},
        tapestry::Tapestry,
        token_type::TokenType,
        weaves::{Weave, Weaver},
    },
    values::{
        Value,
        sign::{SignInfo, SignSchema},
        spell::{SpellInfo, UpValue},
    },
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

#[derive(PartialEq)]
enum Realm {
    Genesis, // script level scope
    Spell,   // spell level scope
}

pub struct WeaveAnalyzer {
    symbol_table: SymbolTable,
    loop_depth: usize,
    weaves_cache: HashMap<String, Weave>,
    spells: HashMap<String, SpellInfo>, // The return weaves of spells
    current_realm: Realm,               // track the realm (scope type) the analyzer is in!
    spell_stack: Vec<String>,           // track the current spell name

    current_upvalues: Vec<UpValue>, // upvalue for currently resolving spell
    current_scope_depth: usize,     // count current the scope depth
    spell_base_depth: usize,        // depth where current spell body starts (parameters live here)
    spell_slot_counter: usize,      // continuous slot counter within current spell

    signs: HashMap<String, SignInfo>,

    parent_map: HashMap<Symbol, Symbol>, // maps a symbol to its parent symbol
}

impl WeaveAnalyzer {
    pub fn new() -> Self {
        let st = SymbolTable::new();
        WeaveAnalyzer {
            symbol_table: st,
            loop_depth: 0,
            weaves_cache: HashMap::new(),
            spells: HashMap::new(),
            current_realm: Realm::Genesis,
            spell_stack: vec![],
            current_upvalues: vec![],
            current_scope_depth: 0,
            spell_base_depth: 0,
            spell_slot_counter: 0,
            parent_map: HashMap::new(),
            signs: HashMap::new(),
        }
    }

    fn error<T>(&self, msg: &str, token: Token) -> Result<T, WeaveError> {
        Err(WeaveError::new(msg, token))
    }

    pub fn analyze(&mut self, ast: Vec<Stmt>) -> WeaveResult<Vec<WovenStmt>> {
        self.analyze_statements(ast)
    }

    fn analyze_statements(&mut self, stmts: Vec<Stmt>) -> WeaveResult<Vec<WovenStmt>> {
        let mut w_stmts: Vec<WovenStmt> = Vec::new();
        for stmt in stmts {
            let woven = self.analyze_statement(stmt)?;
            w_stmts.push(woven);
        }

        Ok(w_stmts)
    }

    fn analyze_statement(&mut self, stmt: Stmt) -> WeaveResult<WovenStmt> {
        match stmt {
            Stmt::Block { statements } => {
                self.symbol_table.new_scope();
                self.current_scope_depth += 1;
                let w_block = self.analyze_statements(statements)?;
                self.current_scope_depth -= 1;
                self.symbol_table.end_scope();
                return Ok(WovenStmt::Block {
                    statements: w_block,
                });
            }
            Stmt::Chant { expression } => {
                let w_expr = self.analyze_expression(expression)?;
                Ok(WovenStmt::Chant { expression: w_expr })
            }
            Stmt::ExprStmt { expr } => {
                let w_expr = self.analyze_expression(expr)?;
                Ok(WovenStmt::ExprStmt { expr: w_expr })
            }
            Stmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => {
                let w_condition = self.analyze_expression(condition)?;

                if !w_condition
                    .weave()
                    .get_tapestry()
                    .has_strand(CONDITIONAL_STRAND)
                {
                    return self.error(
                        "The condition provided to determine the fate does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    );
                }

                // scoping n stuff will be added by the block!
                let w_then = self.analyze_statement(*then_branch)?;
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
            Stmt::VarDeclaration {
                name,
                mutable,
                initializer,
                weave,
            } => {
                // allow variable shadowing from outer scopes
                if let Some(_symbol) = self.symbol_table.resolve_in_current_scope(&name.lexeme) {
                    return self.error(
                        &format!(
                            "The variable '{}' already exists in the current scope!",
                            name.lexeme
                        ),
                        name,
                    );
                }

                let expr_weave: Result<Weave, WeaveError>;
                let w_initializer = match initializer {
                    Some(val) => Some(self.analyze_expression(val)?),
                    None => None,
                };

                let mut parent: Option<Box<Symbol>> = None;

                match &w_initializer {
                    Some(val) => {
                        // Try to get weave from symbol first (for composite weaves like SpellWeave<TextWeave>)
                        expr_weave = if let Some(symbol) = val.symbol() {
                            parent = Some(Box::new(symbol.clone()));
                            Ok(symbol.weave.clone())
                        } else {
                            Ok(val.weave())
                        };
                    }
                    None => {
                        if !mutable {
                            // this shouldnt occur since parser should already have handle this
                            return self.error(
                                "bind values must be initialized with an expression!",
                                name,
                            );
                        }

                        // if no initializer, the weave must be specified. Try to get weave from the specified weave name
                        expr_weave = if weave.is_some() {
                            self.get_weave_from_name(&weave.as_ref().unwrap().base.lexeme)
                                .ok_or(WeaveError::new(
                                    "Couldn't find the weave for the variable! Perhaps you misspelled it?",
                                    name.clone(),
                                ))
                        } else {
                            Err(WeaveError::new(
                                "Couldn't determine a weave for the variable! You shall specify a weave for uninitialized variables!",
                                name.clone(),
                            ))
                        };
                    }
                }

                if weave.is_some() {
                    let base_weave_name = weave.clone().unwrap().base.lexeme;
                    let specified_weave = self.get_weave_from_name(&base_weave_name);

                    if specified_weave.is_none() {
                        return self.error(
                            &format!(
                                "Couldn't find the weave '{}' within the Eira's library!",
                                base_weave_name
                            ),
                            name.clone(),
                        );
                    }
                    let mut specified_weave = specified_weave.unwrap();
                    let inner_weave_of_the_specified_weave = self.get_weave_from_name(&(*weave.unwrap().inner.unwrap().base.lexeme)).unwrap();

                    specified_weave = Weaver::weave(specified_weave, inner_weave_of_the_specified_weave).unwrap();

                    if expr_weave.clone()? != specified_weave {
                        return self.error(
                            &format!("The specified weave for '{}' does not match the weave of the initializer expression!", name.lexeme),
                            name,
                        );
                    }
                }

                let slot = if matches!(self.current_realm, Realm::Spell) {
                    // Inside a spell, use continuous slot counter
                    let current_slot = self.spell_slot_counter;
                    self.spell_slot_counter += 1;
                    current_slot
                } else {
                    // Outside spells, use scope-local slot assignment
                    self.symbol_table.get_current_scope_size()
                };

                let s = self
                    .symbol_table
                    .define(name.lexeme.clone(), expr_weave?, mutable, slot, parent)
                    .unwrap();

                // set a parent relationship if the initializer is a variable
                if let Some(val) = &w_initializer {
                    match val {
                        WovenExpr::Variable { symbol, .. } => {
                            // set parent relationship
                            self.set_parent(s.clone(), symbol.clone());
                        }
                        WovenExpr::Cast {
                            reagents: _,
                            callee: _,
                            weave: _,
                            spell_symbol,
                        } => {
                            // check the returns
                            if let Some(spell) = self.spells.get(&spell_symbol.name) {
                                if let Some(r_symbol) = spell.released_symbol.clone() {
                                    if let Some(parent) = self.find_greatest_parent(&r_symbol) {
                                        self.set_parent(s.clone(), parent.clone());
                                    }
                                }
                            }
                            if s.weave == spell_symbol.weave {
                                // leave it to the gods, idk whats goin on here atp

                                // maybe set parent relationship?
                                // self.set_parent(s.clone(), spell_symbol.clone());
                            }
                        }
                        // WovenExpr::Draw {
                        //     marks: _,
                        //     callee: _,
                        //     tapestry: _,
                        //     sign_info,
                        // } => {
                        //     // check the mark infos
                        //     if let Some(parent) = self.find_greatest_parent(&sign_info.symbol) {
                        //         self.set_parent(s.clone(), parent.clone());
                        //     }
                        // }
                        _ => {}
                    }
                }

                Ok(WovenStmt::VarDeclaration {
                    name: name,
                    mutable: mutable,
                    initializer: w_initializer,
                    symbol: s,
                })
            }
            Stmt::While { condition, body } => {
                let w_condition = self.analyze_expression(condition)?;

                if !w_condition
                    .weave()
                    .get_tapestry()
                    .has_strand(CONDITIONAL_STRAND)
                {
                    return self.error(
                        "The condition provided to determine the fate of loop does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    );
                }

                // enter loop scope (for sever, flow purposes)
                self.loop_depth += 1;

                let w_body = self.analyze_statement(*body)?;

                // loop scope exit
                self.loop_depth -= 1;

                Ok(WovenStmt::While {
                    condition: w_condition,
                    body: Box::new(w_body),
                })
            }
            Stmt::Sever { token } => {
                if self.loop_depth == 0 {
                    return self.error("'sever' cannot be used outside a loop circle!", token);
                }
                Ok(WovenStmt::Sever { token })
            }
            Stmt::Flow { token } => {
                if self.loop_depth == 0 {
                    return self.error("'flow' cannot be used outside a loop circle!", token);
                }
                Ok(WovenStmt::Flow { token })
            }
            Stmt::Release { token, expr } => {
                // Ensure 'release' is only used within a spell realm
                if self.current_realm == Realm::Genesis {
                    return self.error(
                        "Values cannot be released from the 'Genesis' realm!\n\
                        Error: Usage of 'release' outside the spell scope.",
                        token,
                    );
                }

                let curr_spell_name = match self.spell_stack.last() {
                    Some(name) => name.clone(),
                    None => {
                        return self.error("Release used outside of any spell scope.", token);
                    }
                };

                // Ensure spell exists and check if already released
                let spell_entry = match self.spells.get(&curr_spell_name) {
                    Some(v) => v,
                    None => {
                        return self.error(
                            &format!(
                                "No Spell found in the realm with the name '{}'",
                                curr_spell_name
                            ),
                            token,
                        );
                    }
                };

                let expected_weave = spell_entry.release_weave.clone();

                if let Some(e) = expr {
                    let w_expr = self.analyze_expression(e)?;

                    // Try to get the weave from the symbol first (for variables with composite weaves)
                    // Otherwise fall back to tapestry lookup
                    let actual_weave = if let Some(symbol) = w_expr.symbol() {
                        symbol.weave.clone()
                    } else {
                        w_expr.weave()
                            // .ok_or(WeaveError::new(
                            //     "Couldnt find a matching weave for the releasing expression.",
                            //     token.clone(),
                            // ))?
                    };

                    // Exact tapestry check (spells should return the exact weave)
                    if expected_weave != w_expr.weave() {
                        return self.error(
                            &format!(
                                "The spell '{}' was expected to release '{}' but '{}' was released",
                                curr_spell_name,
                                expected_weave.get_name(),
                                actual_weave.get_name()
                            ),
                            token,
                        );
                    }

                    // Record the actual released weave
                    if let Some(v) = self.spells.get_mut(&curr_spell_name) {
                        v.released_weave = Some(actual_weave.clone());
                        v.released_symbol = w_expr.symbol().cloned(); // we gamblin!
                    }

                    Ok(WovenStmt::Release {
                        token: token,
                        expr: Some(w_expr),
                    })
                } else {
                    // release; with no expression implies Emptiness.
                    // If the spell expects a non-empty weave, this is an error.
                    if expected_weave != Weave::Empty {
                        return self.error(
                            &format!(
                                "The spell '{}' expects a value of weave '{}' to be released, but no value was provided.",
                                curr_spell_name, expected_weave.get_name()
                            ),
                            token,
                        );
                    }

                    // Record Emptiness as the released weave
                    if let Some(v) = self.spells.get_mut(&curr_spell_name) {
                        v.released_weave = Some(Weave::Empty);
                    }

                    Ok(WovenStmt::Release {
                        token: token,
                        expr: None,
                    })
                }
            }
            Stmt::Spell {
                name,
                reagents,
                body,
                return_weave,
            } => {
                // allow spell shadowing from outer scopes
                let existing = self.symbol_table.resolve_in_current_scope(&name.lexeme);
                if existing.is_some() {
                    return self.error(
                        &format!(
                            "The spell '{}' already exists in the current scope!",
                            name.lexeme
                        ),
                        name,
                    );
                }

                let mut w_reagents: Vec<WovenReagent> = vec![];
                let slot = self.symbol_table.get_current_scope_size();

                // get the ret type (weave ofcourse)
                let ret_weave = match return_weave {
                    Some(rw) => {
                        let base_weave =
                            self.get_weave_from_name(&rw.base.lexeme)
                                .ok_or(WeaveError::new(
                                    &format!(
                                        "Couldn't find the weave '{}' within the Eira's library!",
                                        rw.base.lexeme
                                    ),
                                    name.clone(),
                                ))?;

                        // TODO: make the check recursive
                        let inner: Option<Weave> = match rw.inner {
                            Some(tkn) => {
                                let w =
                            self.get_weave_from_name(&tkn.base.lexeme)
                                .ok_or(WeaveError::new(
                                    &format!(
                                        "Couldn't find the weave '{}' within the Eira's library!",
                                        tkn.base.lexeme
                                    ),
                                    name.clone(),
                                ))?;

                                Some(w)
                            }
                            None => None,
                        };

                        if inner.is_none() {
                            base_weave
                        } else {
                            let final_weave = Weaver::weave(base_weave, inner.unwrap());
                            if final_weave.is_err() {
                                return self.error(&final_weave.unwrap_err().0, name.clone());
                            }

                            final_weave.unwrap()
                        }
                    }
                    None => Weave::Empty,
                };

                // define the spell
                // Create SpellWeave<ReturnWeave> for the spell's symbol
                let spell_weave = if ret_weave != Weave::Empty {
                    // Spell returns something, so wrap it: SpellWeave<ReturnWeave>
                    Weaver::weave(
                        Weave::Spell {
                            reagents: Vec::new(),
                            release: Box::new(Weave::Empty),
                        },
                        ret_weave.clone(),
                    )
                    .unwrap_or(Weave::Spell {
                        reagents: Vec::new(),
                        release: Box::new(Weave::Empty),
                    })
                } else {
                    // Spell returns nothing, just SpellWeave
                    Weave::Spell {
                        reagents: Vec::new(),
                        release: Box::new(Weave::Empty),
                    }
                };

                let symbol = self
                    .symbol_table
                    .define(name.lexeme.clone(), spell_weave, false, slot, None)
                    .unwrap();

                self.symbol_table.new_scope();

                // spell_base_depth should be equal to depth where spell is defined;
                // so the base_depth should be incremented after savin it
                // Variables from this depth or shallower can be upvalues
                let saved_spell_base_depth = self.spell_base_depth;
                self.spell_base_depth = self.current_scope_depth;

                self.current_scope_depth += 1;

                // Reset spell slot counter for parameters
                self.spell_slot_counter = 0;

                let upvals_saved = std::mem::take(&mut self.current_upvalues);

                for r in reagents {
                    let Some(weave) = self.get_weave_from_name(&r.weave_name.lexeme) else {
                        return self.error(
                            &format!(
                                "Couldn't find the weave '{}' within the Eira's library!",
                                name.lexeme.clone()
                            ),
                            r.weave_name.clone(),
                        );
                    };
                    self.symbol_table.define(
                        r.name.lexeme.clone(),
                        weave.clone(),
                        false,
                        self.spell_slot_counter, // Use continuous slot counter, (lexical scoping doesnt work right here!)
                        None,
                    );
                    self.spell_slot_counter += 1; // Increment for next parameter
                    w_reagents.push(WovenReagent {
                        name: r.name.clone(),
                        weave: weave,
                    });
                }

                // save it...
                self.spells.insert(
                    name.lexeme.clone(),
                    SpellInfo {
                        name: name.lexeme.clone(),
                        reagents: w_reagents.clone(),
                        release_weave: ret_weave.clone(),
                        symbol: symbol.clone(),
                        released_weave: None,
                        released_symbol: None, // we gamblin!
                        upvalues: vec![],
                    },
                );

                self.current_realm = Realm::Spell;
                self.spell_stack.push(name.lexeme.clone());

                let woven_body = self.analyze_statement(*body)?;

                self.spell_stack.pop();

                // Reset spell slot counter when exiting spell
                self.spell_slot_counter = 0;

                // this is inaccurate and buggy. should be replaced. TODO!
                self.current_realm = if self.spell_stack.len() == 0 {
                    Realm::Genesis
                } else {
                    Realm::Spell
                };

                let captured_vals = std::mem::replace(&mut self.current_upvalues, upvals_saved);
                if let Some(s) = self.spells.get_mut(&name.lexeme) {
                    s.upvalues = captured_vals;
                }

                self.current_scope_depth -= 1;
                self.symbol_table.end_scope();

                // Restore base_depth
                self.spell_base_depth = saved_spell_base_depth;

                if let Some(s) = self.spells.get(&name.lexeme) {
                    match s.released_weave.clone() {
                        None => {
                            if s.release_weave == Weave::Empty {
                                // ok, since the expected one was emptyweave and got was none;
                                // which implies a default emptyweave return!
                            } else {
                                return self.error(
                                    &format!(
                                        "The spell '{}' does not release any value, but it was expected to release a value of weave '{}'",
                                        name.lexeme, ret_weave.get_name()
                                    ),
                                    name,
                                );
                            }
                        }
                        Some(rw) => {
                            // this wouldnt really be thrown if the release statment does its job
                            if rw != ret_weave {
                                return self.error(
                                    &format!(
                                        "The spell '{}' was expected to release a value of weave '{}', but it released a value of weave '{}'",
                                        name.lexeme, ret_weave.get_name(), rw.get_name()
                                    ),
                                    name,
                                );
                            }
                        }
                    };

                    Ok(WovenStmt::Spell {
                        name: name,
                        reagents: w_reagents,
                        body: Box::new(woven_body),
                        spell: s.clone(),
                    })
                } else {
                    return self
                        .error("Internal Error: Spell info missing after definition.", name);
                }
            }
            Stmt::Sign { name, marks } => {
                // if self.current_scope_depth != 0 {
                //     return self.error("Signs must be declared at the global scope.", name)
                // }

                if let Some(_) = self.symbol_table.resolve_in_current_scope(&name.lexeme) {
                    return self.error(
                        "A variable has been declared with same name as the sign.",
                        name,
                    );
                }

                if self.signs.contains_key(&name.lexeme) {
                    return self.error("A sign with same name already exists!.", name);
                }

                let slot = self.symbol_table.get_current_scope_size();
                let symbol = self.symbol_table.define(
                    name.lexeme.clone(),
                    Weave::Sign(name.lexeme.clone()),
                    false,
                    slot,
                    None,
                );

                let mut sign_info = SignInfo {
                    schema: SignSchema::new(name.lexeme.clone()),
                    // name: name.lexeme.clone(),
                    marks: HashMap::new(),
                    attunements: HashMap::new(),
                    symbol: symbol.unwrap(),
                };

                let mut names: Vec<String> = vec![];
                let mut w_marks: Vec<WovenMark> = vec![];

                for m in marks {
                    if names.contains(&m.name.lexeme) {
                        return self.error(
                            "A different mark with same name exists in the sign!",
                            m.name,
                        );
                    }
                    names.push(m.name.lexeme.clone());
                    if let Some(mark_weave) = self.get_weave_from_name(&m.weave_name.lexeme) {
                        w_marks.push(WovenMark {
                            name: m.name.clone(),
                            weave: mark_weave.clone(),
                        });

                        sign_info.marks.insert(m.name.lexeme.clone(), mark_weave);
                        sign_info.schema.add_field(m.name.lexeme);
                    } else {
                        return self.error(
                            &format!(
                                "Couldn't find the weave '{}' within the Eira's library!",
                                m.weave_name.lexeme
                            ),
                            m.weave_name,
                        );
                    }
                }

                self.signs.insert(name.lexeme.clone(), sign_info.clone());

                Ok(WovenStmt::Sign {
                    name,
                    marks: w_marks,
                    info: sign_info,
                    // schema
                })
            }
        }
    }

    fn analyze_expression(&mut self, expr: Expr) -> WeaveResult<WovenExpr> {
        match expr {
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let w_left = self.analyze_expression(*left)?;
                let w_right = self.analyze_expression(*right)?;

                if operator.token_type == TokenType::Plus {
                    let left_has_additive =
                        w_left.weave().get_tapestry().has_strand(ADDITIVE_STRAND);
                    let left_has_concat = w_left
                        .weave()
                        .get_tapestry()
                        .has_strand(CONCATINABLE_STRAND);
                    let right_has_additive =
                        w_right.weave().get_tapestry().has_strand(ADDITIVE_STRAND);
                    let right_has_concat = w_right
                        .weave()
                        .get_tapestry()
                        .has_strand(CONCATINABLE_STRAND);

                    // Both must support the same type of operation
                    if (left_has_additive && right_has_additive)
                        || (left_has_concat && right_has_concat)
                    {
                        // Valid operation
                    } else {
                        return self.error(
                            "Cannot perform '+' operation: operands must both contain either 'Additive' or 'Concatinable' strand.",
                            operator,
                        );
                    }
                } else {
                    if let Some(req_strand) = self.strand_from_op(operator.token_type) {
                        if !w_left.weave().get_tapestry().has_strand(req_strand) {
                            return self.error(
                                &format!(
                                    "The weave of one of the operands is not composed of {} strand.",
                                    self.strand_string_from_bits(req_strand)
                                ),
                                operator,
                            );
                        }

                        if !w_right.weave().get_tapestry().has_strand(req_strand) {
                            return self.error(
                                &format!(
                                    "The weave of one of the operands is not composed of {} strand.",
                                    self.strand_string_from_bits(req_strand)
                                ),
                                operator,
                            );
                        }
                    } else {
                        return self.error(
                            &format!("Unknown operation '{}'", operator.lexeme),
                            operator,
                        );
                    }
                }

                let result_weave = match operator.token_type {
                    TokenType::Greater
                    | TokenType::Less
                    | TokenType::EqualEqual
                    | TokenType::LessEqual
                    | TokenType::GreaterEqual
                    | TokenType::BangEqual => Weave::Truth,
                    TokenType::Plus => {
                        // hard coded for now. Should be dynamic later
                        if w_left.weave().get_tapestry().has_strand(ADDITIVE_STRAND)
                            && w_right.weave().get_tapestry().has_strand(ADDITIVE_STRAND)
                        {
                            Weave::Num
                        } else {
                            Weave::Text
                        }
                    }
                    _ => w_left.weave(), // Assumes left-hand side's type
                };

                Ok(WovenExpr::Binary {
                    left: Box::new(w_left),
                    right: Box::new(w_right),
                    operator: operator,
                    weave: result_weave,
                })
            }
            Expr::Grouping { expression } => self.analyze_expression(*expression),
            Expr::Literal { value, token } => {
                let weave = match value {
                    Value::Number(_) => Weave::Num,
                    Value::Emptiness => Weave::Empty,
                    Value::Bool(_) => Weave::Truth,
                    Value::String(_) => Weave::Text,
                    _ => {
                        return self.error("Couldnt find a weave for the value", token.clone());
                    }
                };
                return Ok(WovenExpr::Literal {
                    value: value,
                    token: token,
                    weave,
                });
            }
            Expr::Unary { operand, operator } => {
                if operator.token_type != TokenType::Minus && operator.token_type != TokenType::Bang
                {
                    return self.error("Unknown Unary Operation", operator);
                }
                if let Some(strand) = self.strand_from_op(operator.token_type) {
                    let expr = self.analyze_expression(*operand)?;
                    if !expr.weave().get_tapestry().has_strand(strand) {
                        return self.error(
                            &format!(
                                "The operand does not contain the '{}' strand as required by '{}' operation",
                                self.strand_string_from_bits(strand),
                                operator.lexeme
                            ),
                            operator,
                        );
                    }
                    let weave = expr.weave();
                    Ok(WovenExpr::Unary {
                        operand: Box::new(expr),
                        operator: operator,
                        weave: weave,
                    })
                } else {
                    return self.error("Unknown Operation", operator);
                }
            }
            Expr::Variable { name } => {
                if let Some(symbol) = self.symbol_table.resolve(&name.lexeme).cloned() {
                    //The symbol(variable) has been found
                    self.resolve_n_add_upvalue(symbol.clone());

                    let weave = &symbol.weave;
                    let woven = WovenExpr::Variable {
                        name: name,
                        weave: weave.clone(),
                        symbol: symbol,
                    };

                    Ok(woven)
                } else {
                    return self.error(
                        &format!("'{}' was undefined in the eira-verse!", name.lexeme),
                        name,
                    );
                }
            }
            Expr::Assignment { name, value } => {
                if let Some(resolved) = self.symbol_table.resolve(&name.lexeme).cloned() {
                    if !resolved.mutable {
                        return self.error(
                            "Tried to reassign a value to a 'bind'. Binds cannot be reassigned!",
                            name,
                        );
                    }

                    let woven_expr = self.analyze_expression(*value)?;
                    let weave = woven_expr.weave();

                    // Assignment requires an exact match of the tapestry!
                    if resolved.weave == woven_expr.weave() {
                        match &woven_expr {
                            WovenExpr::Variable { symbol, .. } => {
                                // set parent relationship
                                self.set_parent(resolved.clone(), symbol.clone());
                            }
                            _ => {}
                        }

                        return Ok(WovenExpr::Assignment {
                            name: name,
                            value: Box::new(woven_expr),
                            weave: weave,
                            symbol: resolved,
                        });
                    }

                    return self.error(
                        "The assignee and the value to be assigned are of different Weaves!\nAssignment failed.",
                        name,
                    );
                } else {
                    return self.error(
                        "The mark was no where to be found from this realm!\nVariable resolution failed.",
                        name,
                    );
                }
            }
            Expr::Cast { reagents, callee } => {
                let var_name = &callee.lexeme;

                let Some(symbol) = self.symbol_table.resolve(var_name).cloned() else {
                    return self.error(
                        &format!(
                            "The spell '{}' was not found across the eira realms!",
                            var_name
                        ),
                        callee,
                    );
                };

                let greatest_parent = self.find_greatest_parent(&symbol).unwrap();

                // Check if the spell is compile-time known
                let (spell_info, return_weave) = if greatest_parent != symbol {
                    let spell_name = &greatest_parent.name;
                    let Some(info) = self.spells.get(spell_name).cloned() else {
                        return self.error(
                            &format!(
                                "The spell '{}' was not found across the eira realms!",
                                spell_name
                            ),
                            callee,
                        );
                    };
                    (Some(info.clone()), info.release_weave)
                } else {
                    // this would run if the type isnt predictable at compile time
                    // Check if it's callable using the base tapestry (handles SpellWeave<...> generics)
                    if symbol.weave.get_tapestry().has_strand(CALLABLE_STRAND) {
                        return self.error(
                            &format!(
                                "Cannot cast '{}' since it is not a spell! It has weave '{}'",
                                var_name,
                                symbol.weave.get_name()
                            ),
                            callee,
                        );
                    }

                    if let Some(info) = self.spells.get(&symbol.name).cloned() {
                        (Some(info.clone()), info.release_weave)
                    } else {
                        // the return weave thingy would be unknown at comp time
                        (None, Weave::Empty)
                    }
                };

                // reagent count checks
                if let Some(ref info) = spell_info {
                    if info.reagents.len() != reagents.len() {
                        return self.error(
                            &format!(
                                "The spell '{}' expected {} reagents, but you provided {} of them!",
                                info.name,
                                info.reagents.len(),
                                reagents.len()
                            ),
                            callee,
                        );
                    }
                }

                // Only resolve upvalues for compile-time known spells
                if let Some(ref info) = spell_info {
                    self.resolve_n_add_upvalue(info.symbol.clone());
                }

                let mut w_reagents: Vec<WovenExpr> = vec![];

                // Validate reagent types if we know the spell
                if let Some(ref info) = spell_info {
                    let spell_reagents = info.reagents.clone();
                    for (i, reagent) in reagents.iter().enumerate() {
                        let w_expr = self.analyze_expression(reagent.clone())?;
                        let expected = spell_reagents.get(i).unwrap();
                        if w_expr.weave() != expected.weave {
                            return self.error(
                                &format!(
                                    "The reagent #{} was expected to be {}, but got {}",
                                    i + 1,
                                    expected.weave.get_name(),
                                    w_expr.weave().get_name()
                                ),
                                callee,
                            );
                        }
                        w_reagents.push(w_expr.clone());
                    }
                } else {
                    // runtime validations (VM would be dumb anyways, so this is a gamble)
                    for reagent in reagents.iter() {
                        let w_expr = self.analyze_expression(reagent.clone())?;
                        w_reagents.push(w_expr);
                    }
                }

                Ok(WovenExpr::Cast {
                    reagents: w_reagents,
                    callee: callee,
                    weave: return_weave,
                    spell_symbol: symbol,
                })
            }
            Expr::Draw { marks, callee } => {
                let var_name = &callee.lexeme;

                let Some(_) = self
                    .symbol_table
                    .resolve_in_current_scope(var_name)
                    .cloned()
                else {
                    return self.error(
                        "A variable with the same name as the sign exists in the current scope!",
                        callee,
                    );
                };

                let sign_info = if let Some(info) = self.signs.get(&callee.lexeme) {
                    info.clone()
                } else {
                    return self.error(
                        &format!(
                            "The sign '{}' was not found in the current realm!",
                            callee.lexeme
                        ),
                        callee,
                    );
                };

                // Will have to change for optional fields
                if sign_info.marks.len() != marks.len() {
                    return self.error(
                        &format!(
                            "The sign '{}' expected {} marks, but you provided{} {} of them!",
                            callee.lexeme,
                            sign_info.marks.len(),
                            if marks.len() < sign_info.marks.len() {
                                " only"
                            } else {
                                ""
                            },
                            marks.len()
                        ),
                        callee,
                    );
                }

                let mut w_marks: Vec<WovenEtchedMark> = vec![];
                for mark in marks {
                    let mark_val = self.analyze_expression(mark.expr)?;
                    if let Some(field) = sign_info.marks.get(&mark.name.lexeme) {
                        let mark_weave = mark_val.weave();
                        if *field == mark_weave {
                            w_marks.push(WovenEtchedMark {
                                name: mark.name.clone(),
                                expr: mark_val.clone(),
                            })
                        } else {
                            return self.error(
                                &format!(
                                    "The mark '{}' was expected to have weave '{}' but got '{}'",
                                    mark.name.lexeme,
                                    field.get_name(),
                                    mark_weave.get_name()
                                ),
                                mark.name,
                            );
                        }
                    } else {
                        return self.error(
                            &format!(
                                "The mark '{}' doesn't exist inside {}",
                                mark.name.lexeme, callee.lexeme
                            ),
                            mark.name,
                        );
                    }
                }

                Ok(WovenExpr::Draw {
                    marks: w_marks,
                    callee: callee.clone(),
                    weave: Weave::Sign(sign_info.schema.name.clone()),
                    sign_info: sign_info,
                })
            }
            Expr::Access { material, property } => {
                // it should be a variable expression
                let token = match self.analyze_expression(*material)? {
                    WovenExpr::Variable {
                        name,
                        symbol: _,
                        weave: _,
                    } => name,
                    _ => {
                        return self.error(
                            "Only variables can be accessed with '.' operator!",
                            property,
                        );
                    }
                };

                let Some(symbol) = self.symbol_table.resolve(&token.lexeme).cloned() else {
                    return self.error(
                        &format!(
                            "The mark '{}' was not found across the eira realms!",
                            token.lexeme
                        ),
                        token,
                    );
                };

                let Some(ref sign_symbol) = symbol.parent else {
                    return self.error(
                        &format!(
                            "The mark '{}' is not a material of a sign",
                            token.lexeme.clone()
                        ),
                        token,
                    );
                };

                let sign_name = sign_symbol.name.clone();

                let Some(_) = self.symbol_table.resolve(&sign_name) else {
                    return self.error(
                        &format!(
                            "The sign '{}' was not found across the eira realms!",
                            sign_name
                        ),
                        token,
                    );
                };

                let Some(sign_info) = self.signs.get(&sign_name) else {
                    return self.error(
                        &format!(
                            "The sign '{}' was not found across the eira realms!",
                            sign_name
                        ),
                        token,
                    );
                };

                let Some(mark) = sign_info.schema.get_field_index(property.lexeme.clone()) else {
                    return self.error(
                        &format!(
                            "The mark '{}' is not defined for '{}'",
                            property.lexeme, sign_name
                        ),
                        property,
                    );
                };

                let property_weave = sign_info.marks.get(&property.lexeme);

                if property_weave.is_none() {
                    return self.error(
                        &format!(
                            "Eira couldn't find the weave for property '{}'",
                            property.lexeme
                        ),
                        token,
                    );
                }

                let w_expr = WovenExpr::Variable {
                    name: token.clone(),
                    weave: symbol.weave.clone(),
                    symbol: symbol,
                };

                Ok(WovenExpr::Access {
                    material: Box::new(w_expr),
                    property,
                    field_name_idx: mark as u16,
                    weave: property_weave.unwrap().clone(),
                })
            }
            Expr::Deck { elements, token } => {
                let mut w_elements = vec![];

                let mut prev_elem_weave: Option<Weave> = None;

                if elements.len() > u8::MAX as usize {
                    return self.error("Deck size exceeds the maximum of 255 elements!", token);
                }

                for element in elements {
                    let w_element = self.analyze_expression(element)?;
                    let elem_weave = w_element.weave();
                    if let Some(prev_weave) = prev_elem_weave {
                        if elem_weave != prev_weave {
                            return self
                                .error("All elements of a deck must be of the same weave!", token);
                        }
                    }
                    prev_elem_weave = Some(elem_weave);
                    w_elements.push(w_element);
                }

                // self.symbol_table.define(name, weave, mutable, slot_idx, parent)

                let weave = Weave::Deck(Box::new(prev_elem_weave.unwrap_or(Weave::Empty)));

                Ok(WovenExpr::Deck {
                    elements: w_elements,
                    weave: weave,
                })
            }
            Expr::Extract { deck, index, token } => {
                let w_deck = self.analyze_expression(*deck)?;

                let elem_weave = match w_deck.weave() {
                    Weave::Deck(weave) => *weave,
                    _ => {
                        return self.error(
                            &format!(
                                "'{}' was expected to be a 'Deck' but its not!",
                                w_deck.token().lexeme
                            ),
                            token,
                        );
                    }
                };

                let w_index = self.analyze_expression(*index)?;

                let index_weave = w_index.weave();

                if index_weave != Weave::Num {
                    return self.error(
                        "The index expression of a deck set operation must be of NumWeave!",
                        token.clone(),
                    );
                }

                Ok(WovenExpr::Extract {
                    deck: Box::new(w_deck),
                    index: Box::new(w_index),
                    weave: elem_weave,
                    token,
                })
            }
            Expr::DeckSet {
                deck,
                index,
                value,
                token,
            } => {
                let w_deck = self.analyze_expression(*deck)?;
                let w_index = self.analyze_expression(*index)?;
                let w_value = self.analyze_expression(*value)?;

                let index_weave = w_index.weave();

                if index_weave != Weave::Num {
                    return self.error(
                        "The index expression of a deck set operation must be of NumWeave!",
                        token.clone(),
                    );
                }

                Ok(WovenExpr::DeckSet {
                    deck: Box::new(w_deck),
                    index: Box::new(w_index),
                    value: Box::new(w_value.clone()),
                    weave: w_value.weave(),
                    token,
                })
            }
        }
    }

    /// Get the very first ancestor (greatest parent) of a symbol
    fn find_greatest_parent(&mut self, symbol: &Symbol) -> Option<Symbol> {
        let mut current = symbol.clone();
        while let Some(parent) = self.find_parent(&current) {
            current = parent;
        }
        Some(current)
    }

    /// Get the direct parent of a symbol
    fn find_parent(&self, symbol: &Symbol) -> Option<Symbol> {
        self.parent_map.get(symbol).cloned()
    }

    /// Set the direct parent of a symbol
    fn set_parent(&mut self, child: Symbol, parent: Symbol) {
        self.parent_map.insert(child, parent);
    }

    /// Resolve and add an upvalue for a symbol
    fn resolve_n_add_upvalue(&mut self, symbol: Symbol) {
        // Only capture as upvalue if variable is from the spell's defining scope or outer
        // Parameters and locals have depth greater than the spell base depth
        if self.current_realm == Realm::Spell && symbol.depth <= self.spell_base_depth {
            // check if new. use both index and depth to avoid duplicates
            let is_new = !self
                .current_upvalues
                .iter()
                .any(|it| it.index == symbol.slot_idx && it.depth == symbol.depth);

            if is_new {
                // println!("Added upvalue {:?}", symbol);
                self.current_upvalues.push(UpValue {
                    index: symbol.slot_idx,
                    closed: Value::Emptiness.into(),
                    depth: symbol.depth,
                });
            }
        }
    }

    fn strand_from_op(&self, op: TokenType) -> Option<u64> {
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
    fn strand_string_from_bits(&self, strand: u64) -> &str {
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
            NO_STRAND => "NONE",
            _ => "UNKNOWN",
        }
    }

    fn get_weave_from_name(&mut self, name: &str) -> Option<Weave> {
        match name {
            "Num" => Some(Weave::Num),
            "Text" => Some(Weave::Text),
            "Truth" => Some(Weave::Truth),
            "Empty" => Some(Weave::Empty),
            "Sign" => Some(Weave::Sign(String::new())),
            "Spell" => Some(Weave::Spell {
                reagents: vec![],
                release: Box::new(Weave::Empty),
            }),
            "Deck" => Some(Weave::Deck(Box::new(Weave::Empty))),
            _ => {

                // match user defined types!
                if self.signs.contains_key(name) {
                    Some(Weave::Sign(name.to_owned()))
                } else {
                    None
                }
            },
        }
    }

    // fn get_weave(&self, tapestry: Tapestry) -> Option<Weave> {
    //     match tapestry.0 {
    //         x if x == Weave::Num.get_tapestry().0 => Some(Weave::Num.get_weave()),
    //         x if x == Weave::Text.get_tapestry().0 => Some(Weave::Text.get_weave()),
    //         x if x == Weave::Truth.get_tapestry().0 => Some(Weave::Truth.get_weave()),
    //         x if x == Weave::Spell.get_tapestry().0 => Some(Weave::Spell.get_weave()),
    //         x if x == Weave::Empty.get_tapestry().0 => Some(Weave::Empty.get_weave()),
    //         x if x == Weave::Sign.get_tapestry().0 => Some(Weave::Sign.get_weave()),
    //         x if x == Weave::Deck.get_tapestry().0 => Some(Weave::Deck.get_weave()),
    //         _ => None,
    //     }
    // }
}
