use std::collections::HashMap;

use crate::{
    frontend::{
        expr::{Expr, WovenExpr},
        reagents::WovenReagent,
        scanner::Token,
        stmt::{Stmt, WovenStmt},
        strand::{
            ADDITIVE_STRAND, CALLABLE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND, DIVISIVE_STRAND,
            EQUATABLE_STRAND, INDEXIVE_STRAND, MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND,
            SUBTRACTIVE_STRAND,
        },
        symbol_table::SymbolTable,
        tapestry::Tapestry,
        token_type::TokenType,
        weaves::{EmptyWeave, NumWeave, SpellWeave, TextWeave, TruthWeave, Weave, gen_weave_map},
    },
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

pub struct WeaveAnalyzer {
    symbol_table: SymbolTable,
    loop_depth: usize,
    weaves_cache: HashMap<String, Weave>,
}

impl WeaveAnalyzer {
    pub fn new() -> Self {
        let st = SymbolTable::new();
        WeaveAnalyzer {
            symbol_table: st,
            loop_depth: 0,
            weaves_cache: HashMap::new(),
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
                let w_block = self.analyze_statements(statements)?;
                // let tapestry = w_block.
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
                let depth_before = self.loop_depth;

                let w_body = self.analyze_statement(*body)?;

                // exit the loop scope if not done by sever stmt
                if depth_before < self.loop_depth {
                    self.loop_depth -= 1;
                }

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
                self.loop_depth -= 1;
                Ok(WovenStmt::Sever)
            }

            Stmt::Spell {
                name,
                reagents,
                body,
                return_weave
            } => {
                let mut w_reagents: Vec<WovenReagent> = vec![];
                let slot = self.symbol_table.get_current_scope_size();

                // get the ret type (weave ofc)
                let _ret_weave = match return_weave {
                    Some(rw) => self.get_weave_from_name(&rw)?,
                    None => EmptyWeave,
                };

                // Spells are always SpellWeave (CALLABLE) regardless of return type
                let symbol = self.symbol_table.define(
                    name.lexeme.clone(),
                    SpellWeave,
                    false,
                    slot,
                ).unwrap();

                // Create a new scope for the spell body
                self.symbol_table.new_scope();

                // Define spell parameters as local variables in the new scope
                for r in reagents {
                    let param_weave = self.get_weave_from_name(&r.weave_name)?;
                    let param_slot = self.symbol_table.get_current_scope_size();
                    self.symbol_table.define(
                        r.name.lexeme.clone(),
                        param_weave.clone(),
                        true, // parameters are mutable
                        param_slot,
                    );
                    w_reagents.push(WovenReagent {
                        name: r.name,
                        weave: param_weave,
                    });
                }
                
                let woven_body = self.analyze_statement(*body)?;

                // End the spell scope
                self.symbol_table.end_scope();

                Ok(WovenStmt::Spell {
                    name: name,
                    reagents: w_reagents,
                    body: Box::new(woven_body),
                    symbol: symbol,
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

                let result_tape = match operator.token_type {
                    TokenType::Greater
                    | TokenType::Less
                    | TokenType::EqualEqual
                    | TokenType::LessEqual
                    | TokenType::GreaterEqual
                    | TokenType::BangEqual => TruthWeave.tapestry,
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
                    Value::Spell(_) | Value::Closure(_) => CALLABLE_STRAND,
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
                if let Some(symbol) = self.symbol_table.resolve(&name.lexeme) {
                    //The symbol(variable) has been found
                    let weave = &symbol.weave;
                    let woven = WovenExpr::Variable {
                        name: name,
                        tapestry: weave.tapestry,
                        symbol: symbol.clone(),
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
                        "The mark was no where to be found from this scope!\nVariable resolution failed.",
                        name,
                    ));
                }
            }
            Expr::Call { callee, paren, arguments } => {
                let woven_callee = self.analyze_expression(*callee)?;
                
                // Check if the callee has CALLABLE_STRAND
                if !woven_callee.tapestry().has_strand(CALLABLE_STRAND) {
                    return Err(WeaveError::new(
                        "This value cannot be invoked as a spell!",
                        paren,
                    ));
                }
                
                // Analyze all arguments
                let mut woven_args: Vec<WovenExpr> = vec![];
                for arg in arguments {
                    woven_args.push(self.analyze_expression(arg)?);
                }
                
                // For now, spell calls return EmptyWeave
                // TODO: Track spell return types properly
                let tapestry = EmptyWeave.tapestry;
                
                Ok(WovenExpr::Call {
                    callee: Box::new(woven_callee),
                    paren: paren,
                    arguments: woven_args,
                    tapestry: tapestry,
                })
            }
        }
    }

    fn strand_from_op(&self, op: TokenType) -> Option<u64> {
        match op {
            TokenType::Plus => Some(ADDITIVE_STRAND),
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
        println!("{}", name);

        if let Some(w) = self.weaves_cache.get(name) {
            return Ok(w.clone());
        }

        Err(WeaveError::new(
            "Couldn't find the weave '{}' within the Eira's library!",
            demo_tkn(),
        ))
    }

    fn get_weave(&self, tapestry: Tapestry) -> WeaveResult<Weave> {
        const NUM: u64 = NumWeave.tapestry.0;
        const TEXT: u64 = TextWeave.tapestry.0;
        const TRUTH: u64 = TruthWeave.tapestry.0;
        // println!("{:?}", tapestry);
        match tapestry.0 {
            NUM => Ok(NumWeave),
            TEXT => Ok(TextWeave),
            TRUTH => Ok(TruthWeave),
            _ => Err(WeaveError::new(
                "The tapestries and the weaves were undefined.\nCare to define those weaves?",
                demo_tkn(),
            )),
        }
    }
}
