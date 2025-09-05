use crate::{frontend::{expr::{Expr, WovenExpr}, stmt::{Stmt, WovenStmt}, strands::{ADDITIVE_STRAND, CONCATINABLE_STRAND, CONDITIONAL_STRAND, DIVISIVE_STRAND, EQUATABLE_STRAND, INDEXIVE_STRAND, MULTIPLICATIVE_STRAND, NO_STRAND, ORDINAL_STRAND, SUBTRACTIVE_STRAND}, symbol_table::SymbolTable, tapestry::{self, Tapestry}, token_type::TokenType, weaves::{NumWeave, TextWeave, TruthWeave, Weave}}, runtime::value::Value};
use std::{process::exit};
struct WeaveError(String);

type WeaveResult<T> = Result<T, WeaveError>;



pub struct WeaveAnalyzer {
    symbol_table: SymbolTable,
}

impl WeaveAnalyzer {
    pub fn new() -> Self {
        let st = SymbolTable::new();
        WeaveAnalyzer { symbol_table: st  }
    }
    pub fn anaylze(&mut self, ast: Vec<Stmt>) -> Vec<WovenStmt> {
        match self.analyze_statements(ast) {
            Ok(res) => res,
            Err(err) => {
                println!("Weave Error: {}", err.0);
                exit(169) // exit the program
            }
        }
    }

    fn analyze_statements(&mut self, stmts: Vec<Stmt>) -> WeaveResult<Vec<WovenStmt>>  {
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
                    let w_block = self.analyze_statements(statements)?;
                    // let tapestry = w_block.
                    return Ok(WovenStmt::Block { statements: w_block });
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
                    let w_then = self.analyze_statement(*then_branch)?;
                    let w_else: Option<Box<WovenStmt>> = match else_branch {
                        Some(e_b) => Some(Box::new(self.analyze_statement(*e_b)?)),
                        None => None,
                    };
                    Ok(WovenStmt::Fate { condition: w_condition, then_branch: Box::new(w_then), else_branch: w_else })
                }
                Stmt::VarDeclaration {
                    name,
                    mutable,
                    initializer,
                } => {
                    let weave;
                    let w_initializer = match initializer {
                        Some(val) => Some(self.analyze_expression(val)?),
                        None => None,
                    };

                    match &w_initializer {
                        Some(val) => weave = self.get_weave(val.tapestry()),
                        None => { return Err(WeaveError("Couldnt get a weave for the variable! Perhaps.. ehm try specifying the Weave!".to_owned()));}
                    }

                    self.symbol_table.define(name.lexeme.clone(), weave?, mutable);

                    Ok(WovenStmt::VarDeclaration { name: name, mutable: mutable, initializer: w_initializer })
                }
                Stmt::While { condition, body } => {
                    let w_condition = self.analyze_expression(condition)?;
                    let w_body = self.analyze_statement(*body)?;

                    Ok(WovenStmt::While { condition: w_condition, body: Box::new(w_body) })
                }
                Stmt::Sever => { Ok(WovenStmt::Sever) }
            }
    }

    fn analyze_expression(&mut self, expr: Expr) -> WeaveResult<WovenExpr> {
        match expr {
            Expr::Assignment { name, value } => {
                let w_expr = self.analyze_expression(*value.to_owned())?;
                let tapestry = w_expr.tapestry();
                Ok(WovenExpr::Assignment { name: name, value: Box::new(w_expr), tapestry: tapestry })
            },
            Expr::Binary { left, right, operator } => {
                let w_left = self.analyze_expression(*left)?;
                let w_right = self.analyze_expression(*right)?;

                if let Some(req_strand) = self.strand_from_op(operator.token_type) {
                    if !w_left.tapestry().has_strand(req_strand) {
                        return Err(WeaveError(format!("The weave is not composed of {} strand.", "demoo").to_owned()));
                    }

                     if !w_right.tapestry().has_strand(req_strand) {
                        return Err(WeaveError(format!("The weave is not composed of {} strand.", "demoo").to_owned()));
                    }
                } else {
                    return Err(WeaveError(format!("Unknown operation '{}'", operator.lexeme).to_owned()));
                }

                let result_tape =  match operator.token_type {
                    TokenType::Greater | TokenType::Less | TokenType::EqualEqual | TokenType::BangEqual => {
                        Tapestry::new(CONDITIONAL_STRAND | EQUATABLE_STRAND)
                    }
                    _ => w_left.tapestry() // Assumes left-hand side's type
                };
             
                Ok(WovenExpr::Binary { left: Box::new(w_left), right: Box::new(w_right), operator: operator, tapestry: result_tape })
            },
            Expr::Grouping { expression } => {
                self.analyze_expression(*expression)
            },
            Expr::Literal { value } => {
                let strands = match value {
                    Value::Number(_) => ADDITIVE_STRAND | ORDINAL_STRAND | SUBTRACTIVE_STRAND | DIVISIVE_STRAND | MULTIPLICATIVE_STRAND | EQUATABLE_STRAND,
                    Value::Emptiness => NO_STRAND,
                    Value::Bool(_) => CONDITIONAL_STRAND | EQUATABLE_STRAND,
                    Value::String(_) => CONCATINABLE_STRAND | EQUATABLE_STRAND, // add indexive later,
                    _ => {
                        return Err(WeaveError("Couldnt find a weave for the value".to_owned()));
                    },
                };
                let tapestry = Tapestry::new(strands);
                return Ok(WovenExpr::Literal { value: value, tapestry: tapestry });
                },
            Expr::Unary { operand, operator } => {
                if operator.token_type != TokenType::Minus && operator.token_type != TokenType::Bang {
                    return Err(WeaveError("Unknown Unary Operation".to_owned()));
                }
                if let Some(strand) = self.strand_from_op(operator.token_type) {
                    let expr = self.analyze_expression(*operand)?;
                    if !expr.tapestry().has_strand(strand) {
                        return Err(WeaveError(format!("The operand doesnot contain the strand {}", strand).to_owned()))
                    }
                    let tapestry = expr.tapestry();
                    Ok(WovenExpr::Unary { operand: Box::new(expr), operator: operator, tapestry: tapestry })
                } else {
                    return Err(WeaveError("Unknown Operation".to_owned()))
                }
            },
            Expr::Variable { name } => {
                if let Some(symbol) = self.symbol_table.resolve(&name.lexeme) {
                    //The symbol(variable) has been found
                    let weave = &symbol.weave;
                    let woven = WovenExpr::Variable { name: name, tapestry: weave.tapestry };
                    Ok(woven)
                } else {
                    return Err(WeaveError("Variable resolution failed.".to_owned()))
                }
            },

        }
    }

    fn strand_from_op(&self, op: TokenType) -> Option<u64> {
        match op {
            TokenType::Plus => Some(ADDITIVE_STRAND),
            TokenType::Minus => Some(SUBTRACTIVE_STRAND),
            TokenType::Star => Some(MULTIPLICATIVE_STRAND),
            TokenType::Slash => Some(DIVISIVE_STRAND),
            _ => None
        }
    }

    fn get_weave(&self, tapestry: Tapestry) -> WeaveResult<Weave> {
        const  NUM: u64 = NumWeave.tapestry.0;
        const TEXT: u64 = TextWeave.tapestry.0;
        const TRUTH: u64 = TruthWeave.tapestry.0;
        println!("{:?}",tapestry);
        match tapestry.0 {
            NUM => Ok(NumWeave),
            TEXT => Ok(TextWeave),
            TRUTH => Ok(TruthWeave),
            _ => Err(WeaveError("The tapestries and the weaves were undefined.\nCare to define those weaves?".to_owned()))
        } 
    }
}
