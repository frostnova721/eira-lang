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
        symbol_table::{self, Symbol, SymbolTable},
        tapestry::Tapestry,
        token_type::TokenType,
        weaves::{gen_weave_map, EmptyWeave, NumWeave, SpellWeave, TextWeave, TruthWeave, Weave},
    },
    runtime::spell::{SpellInfo, UpValue},
    value::Value,
};

fn demo_tkn() -> Token {
    Token {
        column: 0,
        lexeme: "idk".to_owned(),
        line: 0,
        token_type: TokenType::Identifier,
    }
}

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

type WeaveResult<T> = Result<T, WeaveError>;

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
                // let tapestry = w_block.
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
                        demo_tkn(),
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
                if let Some(_symbol) = self.symbol_table.resolve(&name.lexeme) {
                    return Err(WeaveError::new(
                        &format!(
                            "The variable '{}' already exists in the current scope!",
                            name.lexeme
                        ),
                        name,
                    ));
                }

                let weave;
                let w_initializer = match initializer {
                    Some(val) => Some(self.analyze_expression(val)?),
                    None => None,
                };

                match &w_initializer {
                    Some(val) => weave = self.get_weave(val.tapestry()),
                    None => {
                        return Err(WeaveError::new(
                            "Couldnt get a weave for the variable! Perhaps.. ehm try specifying the Weave!",
                            name,
                        ));
                    }
                }

                let slot = self.symbol_table.get_current_scope_size();

                let s = self
                    .symbol_table
                    .define(name.lexeme.clone(), weave?, mutable, slot)
                    .unwrap();

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
                        demo_tkn(),
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
            Stmt::Sever => {
                if self.loop_depth == 0 {
                    return Err(WeaveError::new(
                        "'sever' cannot be used outside a loop circle!",
                        demo_tkn(),
                    ));
                }
                Ok(WovenStmt::Sever)
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

                    let actual_weave = self.get_weave(w_expr.tapestry())?;

                    // Exact tapestry check (spells should return the exact weave)
                    if expected_weave.tapestry.0 != w_expr.tapestry().0 {
                        return Err(WeaveError::new(
                            &format!(
                                "The spell '{}' Release Weave '{}' but got '{}'",
                                curr_spell_name, expected_weave.name, actual_weave.name
                            ),
                            token,
                        ));
                    }

                    // Record the actual released weave
                    if let Some(v) = self.spells.get_mut(&curr_spell_name) {
                        v.released_weave = Some(actual_weave.clone());
                    }

                    Ok(WovenStmt::Release {
                        token: token,
                        expr: Some(w_expr),
                    })
                } else {
                    // release; with no expression implies Emptiness.
                    // If the spell expects a non-empty weave, this is an error.
                    if expected_weave.tapestry.0 != EmptyWeave.tapestry.0 {
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
                        v.released_weave = Some(EmptyWeave);
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
                // check n report for existing ones in the scope.
                let existing = self.symbol_table.resolve(&name.lexeme);
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
                    Some(rw) => self.get_weave_from_name(&rw)?,
                    None => EmptyWeave,
                };

                // define the spell
                let symbol = self
                    .symbol_table
                    .define(name.lexeme.clone(), SpellWeave, false, slot)
                    .unwrap();

                self.symbol_table.new_scope();
                self.current_scope_depth += 1;

                let upvals_saved = std::mem::take(&mut self.current_upvalues);

                for r in reagents {
                    let weave = self.get_weave_from_name(&r.weave_name)?;
                    self.symbol_table.define(
                        r.name.lexeme.clone(),
                        weave.clone(),
                        false,
                        self.symbol_table.get_current_scope_size(),
                    );
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
                        upvalues: vec![],
                    },
                );

                self.current_realm = Realm::Spell;
                self.spell_stack.push(name.lexeme.clone());

                let woven_body = self.analyze_statement(*body)?;

                self.spell_stack.pop();
                self.current_realm = if self.spell_stack.len() == 0 {
                    Realm::Genesis
                } else {
                    Realm::Spell
                };

                let captured_vals = std::mem::replace(&mut self.current_upvalues, upvals_saved);
                if let Some(s) = self.spells.get_mut(&name.lexeme) {
                    if !captured_vals.is_empty() {
                        println!(
                            "DEBUG: {} upvalues captured for spell {}",
                            captured_vals.len(),
                            s.name
                        );
                    }
                    s.upvalues = captured_vals;
                }

                self.current_scope_depth -= 1;
                self.symbol_table.end_scope();

                if let Some(s) = self.spells.get(&name.lexeme) {
                    match s.released_weave.clone() {
                        None => {
                            return Err(WeaveError::new(
                                &format!(
                                    "The spell '{}' does not release any value, but it was expected to release a value of weave '{}'",
                                    name.lexeme, ret_weave.name
                                ),
                                name,
                            ));
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
                    | TokenType::BangEqual => TruthWeave.tapestry,
                    TokenType::Plus => {
                        // hard coded for now. Should be dynamic later
                        if w_left.tapestry().has_strand(ADDITIVE_STRAND)
                            && w_right.tapestry().has_strand(ADDITIVE_STRAND)
                        {
                            NumWeave.tapestry
                        } else {
                            TextWeave.tapestry
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
            Expr::Literal { value } => {
                let demo_tkn = Token {
                    column: 0,
                    lexeme: "idk".to_owned(),
                    line: 0,
                    token_type: TokenType::Identifier,
                };
                let strands = match value {
                    Value::Number(_) => NumWeave.tapestry.0,
                    Value::Emptiness => NO_STRAND,
                    Value::Bool(_) => TruthWeave.tapestry.0,
                    Value::String(_) => TextWeave.tapestry.0, // add indexive later,
                    _ => {
                        return Err(WeaveError::new(
                            "Couldnt find a weave for the value",
                            demo_tkn,
                        ));
                    }
                };
                let tapestry = Tapestry::new(strands);
                return Ok(WovenExpr::Literal {
                    value: value,
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
                let spell_name = &callee.lexeme;

                let Some(spell_info) = self.spells.get(spell_name).cloned() else {
                    return Err(WeaveError::new(
                        &format!(
                            "The spell '{}' was not found in the current realm!",
                            spell_name
                        ),
                        callee,
                    ));
                };

                if spell_info.reagents.len() != reagents.len() {
                    return Err(WeaveError::new(
                        &format!(
                            "The spell '{}' expected {} reagents, but you added {} of them!",
                            spell_name.clone(),
                            spell_info.reagents.len(),
                            reagents.len()
                        ),
                        callee,
                    ));
                }

                self.resolve_n_add_upvalue(spell_info.symbol.clone());

                let mut w_reagents: Vec<WovenExpr> = vec![];
                let spell_reagents = spell_info.reagents.clone();
                for (i, reagent) in reagents.iter().enumerate() {
                    let w_expr = self.analyze_expression(reagent.clone())?;
                    let expected = spell_reagents.get(i).unwrap();
                    if w_expr.tapestry().0 != expected.weave.tapestry.0 {
                        return Err(WeaveError::new(
                            &format!(
                                "The reagent #{} for spell '{}' was expected to be {}, but got {}",
                                i + 1,
                                spell_name,
                                expected.weave.name,
                                self.get_weave(w_expr.tapestry())?.name
                            ),
                            callee,
                        ));
                    }
                    w_reagents.push(w_expr.clone());
                }

                Ok(WovenExpr::Cast {
                    reagents: w_reagents,
                    callee: callee,
                    tapestry: spell_info.release_weave.tapestry,
                    spell_symbol: spell_info.symbol,
                })
            }
        }
    }

    fn resolve_n_add_upvalue(&mut self, symbol: Symbol) {
        // if the var resides on a higher scope, its an upvalue..
        if self.current_realm == Realm::Spell && self.current_scope_depth > symbol.depth {
            // check if new
            let is_new = !self.current_upvalues.iter().any(|it| {
                // need some stuff to dectect em...
                it.index == symbol.slot_idx
            });

            if is_new {
                self.current_upvalues.push(UpValue {
                    index: symbol.slot_idx,
                    closed: Value::Emptiness,
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
            TokenType::Bang => Some(CONDITIONAL_STRAND),
            TokenType::Greater
            | TokenType::Less
            | TokenType::GreaterEqual
            | TokenType::LessEqual => Some(ORDINAL_STRAND),
            TokenType::EqualEqual | TokenType::BangEqual => Some(EQUATABLE_STRAND),
            _ => None,
        }
    }

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

    fn get_weave_from_name(&mut self, name: &str) -> WeaveResult<Weave> {
        if self.weaves_cache.is_empty() {
            self.weaves_cache = gen_weave_map();
        }

        if let Some(w) = self.weaves_cache.get(name) {
            return Ok(w.clone());
        }

        Err(WeaveError::new(
            &format!(
                "Couldn't find the weave '{}' within the Eira's library!",
                name
            ),
            demo_tkn(),
        ))
    }

    fn get_weave(&self, tapestry: Tapestry) -> WeaveResult<Weave> {
        const NUM: u64 = NumWeave.tapestry.0;
        const TEXT: u64 = TextWeave.tapestry.0;
        const TRUTH: u64 = TruthWeave.tapestry.0;
        const SPELL: u64 = SpellWeave.tapestry.0;
        // println!("{:?}", tapestry);
        match tapestry.0 {
            NUM => Ok(NumWeave),
            TEXT => Ok(TextWeave),
            TRUTH => Ok(TruthWeave),
            SPELL => Ok(SpellWeave),
            _ => Err(WeaveError::new(
                "The tapestries and the weaves were undefined.\nCare to define those weaves?",
                demo_tkn(),
            )),
        }
    }
}
