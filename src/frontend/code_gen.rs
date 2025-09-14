use std::{collections::HashMap, str, u8};

use crate::{
    assembler::Assembler,
    frontend::{
        expr::WovenExpr,
        scanner::Token,
        stmt::WovenStmt,
        symbol_table::Symbol,
        tapestry::Tapestry,
        token_type::TokenType,
        weaves::{NumWeave, TextWeave, TruthWeave, Weave},
    },
    runtime::{instruction::Instruction},
    value::Value,
};

const NUM: u64 = NumWeave.tapestry.0;
const TEXT: u64 = TextWeave.tapestry.0;
const TRUTH: u64 = TruthWeave.tapestry.0;

#[derive(Debug)]
pub struct GenError {
    pub msg: String,
}

type GenResult<T> = Result<T, GenError>;

fn error(msg: &str) -> GenError {
    GenError {
        msg: msg.to_owned(),
    }
}

pub struct CodeGen {
    woven_ast: Vec<WovenStmt>,
    instructions: Vec<Instruction>,

    register_index: u8,

    constants: Vec<Value>,
    constants_map: HashMap<Value, u16>,
}

impl CodeGen {
    pub fn new(w_ast: Vec<WovenStmt>) -> Self {
        CodeGen {
            woven_ast: w_ast,
            instructions: vec![],
            register_index: 0,
            constants: vec![],
            constants_map: HashMap::new(),
        }
    }

    fn get_next_register(&mut self) -> GenResult<u8> {
        if self.register_index == u8::MAX {
            panic!("Maximum registers allocated! Register overflow?!")
        }
        self.register_index += 1;
        Ok(self.register_index - 1)
    }

    fn get_last_allocated_register(&self) -> u8 {
        self.register_index - 1
    }

    fn add_constant(&mut self, value: Value) -> GenResult<u16> {
        if let Some(val) = self.constants_map.get(&value) {
            return Ok(*val);
        }

        // else add the constant to table and return the index
        let ind = self.constants.len() as u16;
        self.constants.push(value.clone());
        self.constants_map.insert(value, ind);
        Ok(ind)
    }

    fn write_constant(&mut self, value: Value) -> GenResult<u8> {
        let reg = self.get_next_register()?;
        let const_index = self.add_constant(value)?;
        self.instructions.push(Instruction::Constant {
            dest: reg,
            const_index: const_index,
        });
        Ok(reg)
    }

    // Thought this name is fun, nothing else, its the main entry point btw
    pub fn summon_bytecode(&mut self) -> GenResult<Vec<u8>> {
        let stmts = self.woven_ast.clone();

        let _ = self.gen_from_stmts(stmts)?;

        self.instructions.push(Instruction::Halt);

        println!("\n{:?}", self.instructions);

        let bc = Assembler::convert_to_byte_code(&self.instructions);
        Ok(bc) // change later!
    }

    pub fn get_constants(&mut self) -> Vec<Value> {
        self.constants.clone()
    }

    /// A Helper like function to iterate through the statement list
    fn gen_from_stmts(&mut self, stmts: Vec<WovenStmt>) -> GenResult<u8> {
        for stmt in stmts {
            self.gen_from_stmt(stmt)?;
        }
        Ok(0) // dummy result, since statements doesnt care about values produced
    }

    /// Match the type of stmt and generate corresponding instruction 
    fn gen_from_stmt(&mut self, stmt: WovenStmt) -> GenResult<u8> {
        match stmt {
            WovenStmt::ExprStmt { expr } => self.gen_from_expr(expr),

            WovenStmt::VarDeclaration {
                name,
                mutable,
                initializer,
                symbol,
            } => self.gen_var_decl_instruction(name, mutable, initializer, symbol),
            WovenStmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => todo!(),
            WovenStmt::While { condition, body } => todo!(),
            WovenStmt::Chant { expression } => self.gen_chant_stmt(expression),

            WovenStmt::Block { statements } => self.gen_from_stmts(statements),
            WovenStmt::Sever => todo!(),
        }
    }

    fn gen_from_expr(&mut self, expr: WovenExpr) -> GenResult<u8> {
        match expr {
            WovenExpr::Binary {
                left,
                right,
                operator,
                tapestry,
            } => self.gen_binary_instruction(*left, *right, operator, tapestry),
            WovenExpr::Unary {
                operand,
                operator,
                tapestry,
            } => self.gen_unary_instruction(*operand, operator),
            WovenExpr::Literal { value, tapestry } => {
                let val = self.write_constant(value)?;
                Ok(val)
            }
            WovenExpr::Variable {
                name,
                tapestry,
                symbol,
            } => self.gen_variable_instruction(symbol),
            WovenExpr::Grouping {
                expression,
                tapestry,
            } => self.gen_from_expr(*expression),
            WovenExpr::Assignment {
                name,
                value,
                tapestry,
                symbol,
            } => self.gen_assignment_instruction(*value, symbol),
        }
    }

    fn gen_assignment_instruction(&mut self, expr: WovenExpr, symbol: Symbol) -> GenResult<u8> {
        let reg = self.gen_from_expr(expr)?;
        if symbol.depth > 0 {
            self.instructions.push(Instruction::SetLocal {
                src_reg: reg,
                slot_idx: symbol.slot_idx as u16,
            });
        } else {
            let c_ind = self.add_constant(Value::String(symbol.name.into()))?;
            self.instructions.push(Instruction::SetGlobal {
                src_reg: reg,
                const_index: c_ind,
            });
        }
        return Ok(reg);
    }

    /// Checks the depth, sets as local if depth > 0 else as a global with a value if provided.
    fn gen_var_decl_instruction(
        &mut self,
        name: Token,
        mutable: bool,
        initializer: Option<WovenExpr>,
        symbol: Symbol,
    ) -> GenResult<u8> {
        if symbol.depth > 0 {
            if let Some(init) = initializer {
                let src = self.gen_from_expr(init)?;
                self.instructions.push(Instruction::SetLocal {
                    src_reg: src,
                    slot_idx: symbol.slot_idx as u16,
                });
                return Ok(src);
            } else {
                let empty = self.get_next_register()?;
                self.instructions
                    .push(Instruction::Emptiness { dest: empty });
                return Ok(empty);
            }
        } else {
            let c_ind = self.add_constant(Value::String(symbol.name.into()))?;
            if let Some(init) = initializer {
                let src = self.gen_from_expr(init)?;
                self.instructions.push(Instruction::SetGlobal {
                    src_reg: src,
                    const_index: c_ind,
                });
                return Ok(src);
            } else {
                let empty = self.get_next_register()?;
                self.instructions
                    .push(Instruction::Emptiness { dest: empty });
                return Ok(empty);
            }
        }
    }

    fn gen_variable_instruction(&mut self, symbol: Symbol) -> GenResult<u8> {
        let dest = self.get_next_register()?;

        if symbol.depth > 0 {
            self.instructions.push(Instruction::GetLocal {
                dest: dest,
                slot_index: symbol.slot_idx as u16,
            });
        } else {
            let const_idx = self.add_constant(Value::String(symbol.name.into()))?;
            self.instructions.push(Instruction::GetGlobal {
                dest: dest,
                const_index: const_idx,
            });
        }
        Ok(dest)
    }

    fn gen_unary_instruction(&mut self, operand: WovenExpr, op: Token) -> GenResult<u8> {
        let register = self.gen_from_expr(operand)?;
        let dest = self.get_next_register()?;

        match op.token_type {
            TokenType::Minus => {
                self.instructions.push(Instruction::Negate { dest: dest, r1: register });
                Ok(dest)
            }
            TokenType::Bang =>{
                self.instructions.push(Instruction::Not { dest: dest, r1: register });
                Ok(dest)
            },
            _ => {
                // This error msg should be shown to the user, if it does, compiler is bugged
                return Err(error(&format!(
                    "Strand for '{}' operation hasnt been entangled with Eira realms!.\nThis error shouldn't be thrown, Report it to devs!",
                    op.lexeme
                )));
            }
        }
    }

    fn gen_binary_instruction(
        &mut self,
        left: WovenExpr,
        right: WovenExpr,
        op: Token,
        tapestry: Tapestry,
    ) -> GenResult<u8> {
        // generate left
        let r1 = self.gen_from_expr(left.clone())?;

        //generate right
        let r2 = self.gen_from_expr(right.clone())?;

        let reg = match self.get_weave(tapestry)?.tapestry.0 {
            NUM => self.gen_num_op(r1, r2, op),
            _ => return Err(error("Unknown weave brotha, check it.")),
        }?;
        return Ok(reg);
    }

    fn gen_chant_stmt(&mut self, expr: WovenExpr) -> GenResult<u8> {
        let expression = self.gen_from_expr(expr)?;
        let inst = Instruction::Print { r1: expression };
        self.instructions.push(inst);
        Ok(expression)
    }

    fn gen_num_op(&mut self, left: u8, right: u8, op: Token) -> GenResult<u8> {
        match op.token_type {
            TokenType::Plus => {
                let dest_reg = self.get_next_register()?;
                let add = Instruction::Add {
                    dest: dest_reg,
                    r1: left,
                    r2: right,
                };
                self.instructions.push(add);
            }
            TokenType::Minus => {
                let dest_reg = self.get_next_register()?;
                let sub = Instruction::Subtract {
                    dest: dest_reg,
                    r1: left,
                    r2: right,
                };
                self.instructions.push(sub);
            }
            TokenType::Slash => {
                let dest_reg = self.get_next_register()?;
                let slash = Instruction::Divide {
                    dest: dest_reg,
                    r1: left,
                    r2: right,
                };
                self.instructions.push(slash);
            }
            TokenType::Star => {
                let dest_reg = self.get_next_register()?;
                let mul = Instruction::Multiply {
                    dest: dest_reg,
                    r1: left,
                    r2: right,
                };
                self.instructions.push(mul);
            }
            _ => {
                 // This error msg should be shown to the user, if it does, compiler is bugged
                return Err(error(&format!(
                    "Strand for '{}' operation hasnt been entangled with Eira realms!.\nThis error shouldn't be thrown, Report it to devs!",
                    op.lexeme
                )));
            }
        }
        Ok(self.get_last_allocated_register())
    }

    fn get_weave(&self, tapestry: Tapestry) -> GenResult<Weave> {
        // println!("{:?}", tapestry);
        match tapestry.0 {
            NUM => Ok(NumWeave),
            TEXT => Ok(TextWeave),
            TRUTH => Ok(TruthWeave),
            _ => {
                // let demo_tkn = Token {
                //     column: 0,
                //     lexeme: "idk".to_owned(),
                //     line: 0,
                //     token_type: TokenType::Identifier,
                // };
                Err(error(
                    "The tapestries and the weaves were undefined.\nCare to define those weaves?",
                    // demo_tkn,
                ))
            }
        }
    }
}

// pub trait StrandBehaviour {
//     fn can_binary(&self) -> bool;
//     fn can_unary(&self) -> bool;
//     fn gen_binary_instruction(&mut self, left: WovenExpr, right: WovenExpr, op: Token) -> GenResult<Instruction>;
//     fn gen_unary_instruction(&mut self, operand: WovenExpr, op: Token) -> GenResult<Instruction>;
// }

// pub struct SubtractBehaviour {}
// pub struct AdditiveBehaviour {}
// pub struct MultiplicativeBehaviour {}
// pub struct DivisiveBehaviour {}
