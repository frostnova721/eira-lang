use std::{collections::HashMap, str, u8};

use crate::{
    frontend::{
        expr::WovenExpr,
        scanner::Token,
        stmt::WovenStmt,
        tapestry::Tapestry,
        token_type::{self, TokenType},
        weaves::{NumWeave, TextWeave, TruthWeave, Weave},
    },
    runtime::{instruction::{self, Instruction}, value::Value},
};

const NUM: u64 = NumWeave.tapestry.0;
const TEXT: u64 = TextWeave.tapestry.0;
const TRUTH: u64 = TruthWeave.tapestry.0;

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
    bytecode: Vec<u8>,
    instructions: Vec<Instruction>,

    register_index: u8,

    constants: Vec<Value>,
    constants_map: HashMap<Value, u16>
}

impl CodeGen {
    pub fn new(w_ast: Vec<WovenStmt>) -> Self {
        CodeGen {
            woven_ast: w_ast,
            bytecode: vec![],
            instructions: vec![],
            register_index: 0,
            constants: vec![],
            constants_map: HashMap::new(),
        }
    }

    fn get_next_register(&mut self) -> GenResult<u8> {
        if self.register_index+1 > u8::MAX {
            return Err(error("Out of magical cauldrons!"));
        }

        self.register_index += 1;
        Ok(self.register_index-1)
    }

    fn add_constant(&mut self, value: Value) -> GenResult<u16> {
        // let val = self.constants_map.get(&value); // later it seems
        self.constants.push(value);
        Ok((self.constants.len()-1) as u16)
    }

    fn write_constant(&mut self, value: Value) -> GenResult<u8> {
         let reg = self.get_next_register()?;
         let const_index = self.add_constant(value)?;
        self.instructions.push(Instruction::Constant { dest: reg, const_index: const_index });
        Ok(reg)
    }

    // Thought this name is fun, nothing else, its the main entry point btw
    pub fn summon_bytecode(&mut self) {
        let stmts = self.woven_ast.clone();
        for stmt in stmts {
            self.gen_from_stmt(stmt);
        }
        println!("{:?}", self.instructions);
    }

    fn gen_from_stmt(&mut self, stmt: WovenStmt) {
        match stmt {
            WovenStmt::ExprStmt { expr } => {
                self.gen_from_expr(expr);
            }
            WovenStmt::VarDeclaration {
                name,
                mutable,
                initializer,
            } => todo!(),
            WovenStmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => todo!(),
            WovenStmt::While { condition, body } => todo!(),
            WovenStmt::Chant { expression } => todo!(),
            WovenStmt::Block { statements } => todo!(),
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
            } => {
                self.gen_binary_instruction(*left, *right, operator, tapestry)
            }
            WovenExpr::Unary {
                operand,
                operator,
                tapestry,
            } => todo!(),
            WovenExpr::Literal { value, tapestry } => {
                let val = self.write_constant(value)?;
                Ok(val)
            },
            WovenExpr::Variable { name, tapestry } => todo!(),
            WovenExpr::Grouping {
                expression,
                tapestry,
            } => todo!(),
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

    fn gen_num_op(&mut self, left: u8, right: u8, op: Token) -> GenResult<u8> {
        match op.token_type {
            TokenType::Plus => {
                let dest_reg = self.get_next_register()?;
                let add = Instruction::Add { dest: dest_reg, r1: left, r2: right };
                self.instructions.push(add);
            }
            TokenType::Minus => {}
            TokenType::Slash => {}
            TokenType::Star => {}
            _ => {
                return Err(error(&format!(
                    "Strand for '{}' operation hasnt been entangled with Eira realms!",
                    op.lexeme
                )));
            }
        }
        Ok(self.get_next_register()?)
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
