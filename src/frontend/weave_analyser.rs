use std::collections::HashMap;

use crate::{
    frontend::{
        expr::{Expr, WovenExpr},
        reagents::WovenReagent,
        scanner::Token,
        stmt::{Stmt, WovenStmt},
        strand::{
            ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND,
            DIVISIVE_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, ITERABLE_STRAND,
            MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND,
        },
        symbol_table::{Symbol, SymbolTable},
        tapestry::Tapestry,
        token_type::TokenType,
        weaves::{Weave, Weaver, Weaves, gen_weave_map},
    },
    values::Value,
    values::spell::{SpellInfo, UpValue},
};

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
        }
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

                if !w_condition.tapestry().has_strand(CONDITIONAL_STRAND) {
                    return Err(WeaveError::new(
                        "The condition provided to determine the fate does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    ));
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
            } => {
                // allow variable shadowing from outer scopes
                if let Some(_symbol) = self.symbol_table.resolve_in_current_scope(&name.lexeme) {
                    return Err(WeaveError::new(
                        &format!(
                            "The variable '{}' already exists in the current scope!",
                            name.lexeme
                        ),
                        name,
                    ));
                }

                let weave: Result<Weave, WeaveError>;
                let w_initializer = match initializer {
                    Some(val) => Some(self.analyze_expression(val)?),
                    None => None,
                };

                match &w_initializer {
                    Some(val) => {
                        // Try to get weave from symbol first (for composite weaves like SpellWeave<TextWeave>)
                        weave = if let Some(symbol) = val.symbol() {
                            Ok(symbol.weave.clone())
                        } else {
                            self.get_weave(val.tapestry()).ok_or(WeaveError::new(
                                "Couldnt get a weave for the variable! Perhaps.. ehm try specifying the Weave!",
                                name.clone(),
                            ))
                        };
                    }
                    None => {
                        return Err(WeaveError::new(
                            "Couldnt get a weave for the variable! Perhaps.. ehm try specifying the Weave!",
                            name,
                        ));
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
                    .define(name.lexeme.clone(), weave?, mutable, slot)
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
                            tapestry: _,
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
                            if s.weave.tapestry.0 == spell_symbol.weave.tapestry.0 {
                                // leave it to the gods, idk whats goin on here atp

                                // maybe set parent relationship?
                                // self.set_parent(s.clone(), spell_symbol.clone());
                            }
                        }
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

                if !w_condition.tapestry().has_strand(CONDITIONAL_STRAND) {
                    return Err(WeaveError::new(
                        "The condition provided to determine the fate of loop does not contain the 'Conditional' strand.",
                        w_condition.token(),
                    ));
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
                    return Err(WeaveError::new(
                        "'sever' cannot be used outside a loop circle!",
                        token,
                    ));
                }
                Ok(WovenStmt::Sever { token })
            }
            Stmt::Flow { token } => {
                if self.loop_depth == 0 {
                    return Err(WeaveError::new(
                        "'flow' cannot be used outside a loop circle!",
                        token,
                    ));
                }
                Ok(WovenStmt::Flow { token })
            }
            Stmt::Release { token, expr } => {
                // Ensure 'release' is only used within a spell realm
                if self.current_realm == Realm::Genesis {
                    return Err(WeaveError::new(
                        "Values cannot be released from the 'Genesis' realm!\n\
                        Error: Usage of 'release' outside the spell scope.",
                        token,
                    ));
                }

                let curr_spell_name = match self.spell_stack.last() {
                    Some(name) => name.clone(),
                    None => {
                        return Err(WeaveError::new(
                            "Release used outside of any spell scope.",
                            token,
                        ));
                    }
                };

                // Ensure spell exists and check if already released
                let spell_entry = match self.spells.get(&curr_spell_name) {
                    Some(v) => v,
                    None => {
                        return Err(WeaveError::new(
                            &format!(
                                "No Spell found in the realm with the name '{}'",
                                curr_spell_name
                            ),
                            token,
                        ));
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
                        self.get_weave(w_expr.tapestry()).ok_or(WeaveError::new(
                            "Couldnt find a matching weave for the releasing expression.",
                            token.clone(),
                        ))?
                    };

                    // Exact tapestry check (spells should return the exact weave)
                    if expected_weave.tapestry.0 != w_expr.tapestry().0 {
                        return Err(WeaveError::new(
                            &format!(
                                "The spell '{}' was expected to release '{}' but '{}' was released",
                                curr_spell_name, expected_weave.name, actual_weave.name
                            ),
                            token,
                        ));
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
                    if expected_weave.tapestry.0 != Weaves::EmptyWeave.get_weave().tapestry.0 {
                        return Err(WeaveError::new(
                            &format!(
                                "The spell '{}' expects a value of weave '{}' to be released, but no value was provided.",
                                curr_spell_name, expected_weave.name
                            ),
                            token,
                        ));
                    }

                    // Record Emptiness as the released weave
                    if let Some(v) = self.spells.get_mut(&curr_spell_name) {
                        v.released_weave = Some(Weaves::EmptyWeave.get_weave());
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
                    return Err(WeaveError::new(
                        &format!(
                            "The spell '{}' already exists in the current scope!",
                            name.lexeme
                        ),
                        name,
                    ));
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

                        let inner: Option<Weave> = match rw.inner {
                            Some(tkn) => {
                                let w =
                            self.get_weave_from_name(&tkn.lexeme)
                                .ok_or(WeaveError::new(
                                    &format!(
                                        "Couldn't find the weave '{}' within the Eira's library!",
                                        tkn.lexeme
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
                                return Err(WeaveError::new(
                                    &final_weave.unwrap_err().0,
                                    name.clone(),
                                ));
                            }

                            final_weave.unwrap()
                        }
                    }
                    None => Weaves::EmptyWeave.get_weave(),
                };

                // define the spell
                // Create SpellWeave<ReturnWeave> for the spell's symbol
                let spell_weave = if ret_weave.name != "EmptyWeave" {
                    // Spell returns something, so wrap it: SpellWeave<ReturnWeave>
                    Weaver::weave(Weaves::SpellWeave.get_weave(), ret_weave.clone())
                        .unwrap_or(Weaves::SpellWeave.get_weave())
                } else {
                    // Spell returns nothing, just SpellWeave
                    Weaves::SpellWeave.get_weave()
                };

                let symbol = self
                    .symbol_table
                    .define(name.lexeme.clone(), spell_weave, false, slot)
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
                        return Err(WeaveError::new(
                            &format!(
                                "Couldn't find the weave '{}' within the Eira's library!",
                                name.lexeme.clone()
                            ),
                            r.weave_name.clone(),
                        ));
                    };
                    self.symbol_table.define(
                        r.name.lexeme.clone(),
                        weave.clone(),
                        false,
                        self.spell_slot_counter, // Use continuous slot counter, (lexical scoping doesnt work right here!)
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
                            if s.release_weave == Weaves::EmptyWeave.get_weave() {
                                // ok, since the expected one was emptyweave and got was none;
                                // which implies a default emptyweave return!
                            } else {
                                return Err(WeaveError::new(
                                    &format!(
                                        "The spell '{}' does not release any value, but it was expected to release a value of weave '{}'",
                                        name.lexeme, ret_weave.name
                                    ),
                                    name,
                                ));
                            }
                        }
                        Some(rw) => {
                            // this wouldnt really be thrown if the release statment does its job
                            if rw.tapestry.0 != ret_weave.tapestry.0 {
                                return Err(WeaveError::new(
                                    &format!(
                                        "The spell '{}' was expected to release a value of weave '{}', but it released a value of weave '{}'",
                                        name.lexeme, ret_weave.name, rw.name
                                    ),
                                    name,
                                ));
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
                    return Err(WeaveError::new(
                        "Internal Error: Spell info missing after definition.",
                        name,
                    ));
                }
            }
            Stmt::Sign { name, marks } => todo!(),
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
                    let left_has_additive = w_left.tapestry().has_strand(ADDITIVE_STRAND);
                    let left_has_concat = w_left.tapestry().has_strand(CONCATINABLE_STRAND);
                    let right_has_additive = w_right.tapestry().has_strand(ADDITIVE_STRAND);
                    let right_has_concat = w_right.tapestry().has_strand(CONCATINABLE_STRAND);

                    // Both must support the same type of operation
                    if (left_has_additive && right_has_additive)
                        || (left_has_concat && right_has_concat)
                    {
                        // Valid operation
                    } else {
                        return Err(WeaveError::new(
                            "Cannot perform '+' operation: operands must both contain either 'Additive' or 'Concatinable' strand.",
                            operator,
                        ));
                    }
                } else {
                    if let Some(req_strand) = self.strand_from_op(operator.token_type) {
                        if !w_left.tapestry().has_strand(req_strand) {
                            return Err(WeaveError::new(
                                &format!(
                                    "The weave of one of the operands is not composed of {} strand.",
                                    self.strand_string_from_bits(req_strand)
                                ),
                                operator,
                            ));
                        }

                        if !w_right.tapestry().has_strand(req_strand) {
                            return Err(WeaveError::new(
                                &format!(
                                    "The weave of one of the operands is not composed of {} strand.",
                                    self.strand_string_from_bits(req_strand)
                                ),
                                operator,
                            ));
                        }
                    } else {
                        return Err(WeaveError::new(
                            &format!("Unknown operation '{}'", operator.lexeme),
                            operator,
                        ));
                    }
                }

                let result_tape = match operator.token_type {
                    TokenType::Greater
                    | TokenType::Less
                    | TokenType::EqualEqual
                    | TokenType::LessEqual
                    | TokenType::GreaterEqual
                    | TokenType::BangEqual => Weaves::TruthWeave.get_weave().tapestry,
                    TokenType::Plus => {
                        // hard coded for now. Should be dynamic later
                        if w_left.tapestry().has_strand(ADDITIVE_STRAND)
                            && w_right.tapestry().has_strand(ADDITIVE_STRAND)
                        {
                            Weaves::NumWeave.get_weave().tapestry
                        } else {
                            Weaves::TextWeave.get_weave().tapestry
                        }
                    }
                    _ => w_left.tapestry(), // Assumes left-hand side's type
                };

                Ok(WovenExpr::Binary {
                    left: Box::new(w_left),
                    right: Box::new(w_right),
                    operator: operator,
                    tapestry: result_tape,
                })
            }
            Expr::Grouping { expression } => self.analyze_expression(*expression),
            Expr::Literal { value, token } => {
                let strands = match value {
                    Value::Number(_) => Weaves::NumWeave.get_weave().tapestry.0,
                    Value::Emptiness => NO_STRAND,
                    Value::Bool(_) => Weaves::TruthWeave.get_weave().tapestry.0,
                    Value::String(_) => Weaves::TextWeave.get_weave().tapestry.0, // add indexive later,
                    _ => {
                        return Err(WeaveError::new(
                            "Couldnt find a weave for the value",
                            token.clone(),
                        ));
                    }
                };
                let tapestry = Tapestry::new(strands);
                return Ok(WovenExpr::Literal {
                    value: value,
                    token: token,
                    tapestry: tapestry,
                });
            }
            Expr::Unary { operand, operator } => {
                if operator.token_type != TokenType::Minus && operator.token_type != TokenType::Bang
                {
                    return Err(WeaveError::new("Unknown Unary Operation", operator));
                }
                if let Some(strand) = self.strand_from_op(operator.token_type) {
                    let expr = self.analyze_expression(*operand)?;
                    if !expr.tapestry().has_strand(strand) {
                        return Err(WeaveError::new(
                            &format!(
                                "The operand does not contain the '{}' strand as required by '{}' operation",
                                self.strand_string_from_bits(strand),
                                operator.lexeme
                            ),
                            operator,
                        ));
                    }
                    let tapestry = expr.tapestry();
                    Ok(WovenExpr::Unary {
                        operand: Box::new(expr),
                        operator: operator,
                        tapestry: tapestry,
                    })
                } else {
                    return Err(WeaveError::new("Unknown Operation", operator));
                }
            }
            Expr::Variable { name } => {
                if let Some(symbol) = self.symbol_table.resolve(&name.lexeme).cloned() {
                    //The symbol(variable) has been found
                    self.resolve_n_add_upvalue(symbol.clone());

                    let weave = &symbol.weave;
                    let woven = WovenExpr::Variable {
                        name: name,
                        tapestry: weave.tapestry,
                        symbol: symbol,
                    };

                    Ok(woven)
                } else {
                    return Err(WeaveError::new("Variable resolution failed.", name));
                }
            }
            Expr::Assignment { name, value } => {
                if let Some(resolved) = self.symbol_table.resolve(&name.lexeme).cloned() {
                    if !resolved.mutable {
                        return Err(WeaveError::new(
                            "Tried to reassign a value to a 'bind'. Binds cannot be reassigned!",
                            name,
                        ));
                    }

                    let woven_expr = self.analyze_expression(*value)?;
                    let tapestry = woven_expr.tapestry();

                    // Assignment requires an exact match of the tapestry!
                    if resolved.weave.tapestry.0 == woven_expr.tapestry().0 {
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
                            tapestry: tapestry,
                            symbol: resolved,
                        });
                    }

                    return Err(WeaveError::new(
                        "The assignee and the value to be assigned are of different Weaves!\nAssignment failed.",
                        name,
                    ));
                } else {
                    return Err(WeaveError::new(
                        "The mark was no where to be found from this realm!\nVariable resolution failed.",
                        name,
                    ));
                }
            }
            Expr::Cast { reagents, callee } => {
                let var_name = &callee.lexeme;

                let Some(symbol) = self.symbol_table.resolve(var_name).cloned() else {
                    return Err(WeaveError::new(
                        &format!(
                            "The spell '{}' was not found in the current realm!",
                            var_name
                        ),
                        callee,
                    ));
                };

                let greatest_parent = self.find_greatest_parent(&symbol).unwrap();

                // Check if the spell is compile-time known
                let (spell_info, return_tapestry) = if greatest_parent != symbol {
                    let spell_name = &greatest_parent.name;
                    let Some(info) = self.spells.get(spell_name).cloned() else {
                        return Err(WeaveError::new(
                            &format!(
                                "The spell '{}' was not found in the current realm!",
                                spell_name
                            ),
                            callee,
                        ));
                    };
                    (Some(info.clone()), info.release_weave.tapestry)
                } else {
                    // this would run if the type isnt predictable at compile time
                    // Check if it's callable using the base tapestry (handles SpellWeave<...> generics)
                    if symbol.weave.base_tapestry != Tapestry::new(CALLABLE_STRAND) {
                        return Err(WeaveError::new(
                            &format!(
                                "Cannot cast '{}' since it is not a spell! It has weave '{}'",
                                var_name, symbol.weave.name
                            ),
                            callee,
                        ));
                    }

                    if let Some(info) = self.spells.get(&symbol.name).cloned() {
                        (Some(info.clone()), info.release_weave.tapestry)
                    } else {
                        // the return weave thingy would be unknown at comp time
                        (None, Tapestry::new(NO_STRAND))
                    }
                };

                // reagent count checks
                if let Some(ref info) = spell_info {
                    if info.reagents.len() != reagents.len() {
                        return Err(WeaveError::new(
                            &format!(
                                "The spell expected {} reagents, but you provided {} of them!",
                                info.reagents.len(),
                                reagents.len()
                            ),
                            callee,
                        ));
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
                        if w_expr.tapestry().0 != expected.weave.tapestry.0 {
                            return Err(WeaveError::new(
                                &format!(
                                    "The reagent #{} was expected to be {}, but got {}",
                                    i + 1,
                                    expected.weave.name,
                                    self.get_weave(w_expr.tapestry()).unwrap().name
                                ),
                                callee,
                            ));
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
                    tapestry: return_tapestry,
                    spell_symbol: symbol,
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
        if self.weaves_cache.is_empty() {
            self.weaves_cache = gen_weave_map();
        }

        self.weaves_cache.get(name).cloned()
    }

    fn get_weave(&self, tapestry: Tapestry) -> Option<Weave> {
        match tapestry.0 {
            x if x == Weaves::NumWeave.get_weave().tapestry.0 => Some(Weaves::NumWeave.get_weave()),
            x if x == Weaves::TextWeave.get_weave().tapestry.0 => {
                Some(Weaves::TextWeave.get_weave())
            }
            x if x == Weaves::TruthWeave.get_weave().tapestry.0 => {
                Some(Weaves::TruthWeave.get_weave())
            }
            x if x == Weaves::SpellWeave.get_weave().tapestry.0 => {
                Some(Weaves::SpellWeave.get_weave())
            }
            _ => None,
        }
    }
}
