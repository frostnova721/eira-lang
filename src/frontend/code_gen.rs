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
    runtime::instruction::Instruction,
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

struct LoopBlock {
    severs: Vec<usize>,
    flows: Vec<usize>,
}

pub struct CodeGen {
    woven_ast: Vec<WovenStmt>,
    instructions: Vec<Instruction>,

    register_index: u8,

    constants: Vec<Value>,
    constants_map: HashMap<Value, u16>,

    loop_blocks: Vec<LoopBlock>,
}

impl CodeGen {
    pub fn new(w_ast: Vec<WovenStmt>) -> Self {
        CodeGen {
            woven_ast: w_ast,
            instructions: vec![],
            register_index: 0,
            constants: vec![],
            constants_map: HashMap::new(),
            loop_blocks: vec![],
        }
    }

    //--------------- Helpers ---------------

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

    fn write_jump(&mut self, inst: Instruction) -> usize {
        self.instructions.push(inst);
        self.instructions.len() - 1
    }

    fn patch_jump(&mut self, jump_idx: usize) -> GenResult<()> {
        let mut offset = 0;

        for i in (jump_idx + 1)..self.instructions.len() {
            offset += self.instructions[i].len();
        }

        if offset > u16::MAX as usize {
            return Err(error("The magic is too complex(long) to jump over!"));
        }

        match &mut self.instructions[jump_idx] {
            Instruction::JumpIfFalse { offset: o, .. } => *o = offset as u16,
            Instruction::Jump { offset: o } => *o = offset as u16,
            _ => {
                return Err(error(
                    "Hmmm... this error shouldnt be thrown! If you are encountering this, congrats! I see a good future in you.",
                ));
            }
        }

        Ok(())
    }

    fn write_loop(&mut self, start: usize) -> GenResult<()> {
        let body_bytes_size: usize = self.instructions[start..].iter().map(|inst| inst.len()).sum();

        // 2 byte for u16's size, 1 for opcode (2+1=3 iydk)
        let total_offset = body_bytes_size + 3;

        if total_offset > u16::MAX as usize {
            return Err(error("Loop Jump Offset exceeds the 2byte limit."));
        }

        self.instructions.push(Instruction::Loop { offset: total_offset as u16 });
        Ok(())
    }

    //--------------- Interface/ Public fns ---------------

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

    //--------------- Actual Core parts ---------------

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
            } => self.gen_fate_instructions(condition, *then_branch, else_branch),
            WovenStmt::While { condition, body } => self.gen_while_instructions(condition, *body),
            WovenStmt::Chant { expression } => self.gen_chant_stmt(expression),

            WovenStmt::Block { statements } => self.gen_from_stmts(statements),
            WovenStmt::Sever => self.gen_sever_instructions(),
        }
    }

    fn gen_sever_instructions(&mut self) -> GenResult<u8> {
         if self.loop_blocks.is_empty() {
            return Err(error("Only the loops can be severed."));
        }
        let ind = self.write_jump(Instruction::Jump { offset: 0xffff });
        self.loop_blocks.last_mut().unwrap().severs.push(ind);

        // dummy return
        Ok(self.register_index)
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

    fn gen_while_instructions(&mut self, condition: WovenExpr, body: WovenStmt) -> GenResult<u8> {
        let start = self.instructions.len();
        let cond_reg = self.gen_from_expr(condition)?;

        let exit = self.write_jump(Instruction::JumpIfFalse {
            condition_reg: cond_reg,
            offset: 0xffff,
        });

        // Add a loop block before the body to manipulate iteration incase of severs or flows
        self.loop_blocks.push(LoopBlock {
            severs: vec![],
            flows: vec![],
        });

        self.gen_from_stmt(body)?;

        self.write_loop(start)?;

        self.patch_jump(exit)?;

        let severs = self.loop_blocks.pop().unwrap().severs;

        for jump in severs {
            self.patch_jump(jump)?;
        }

        Ok(cond_reg)
    }

    fn gen_fate_instructions(
        &mut self,
        condition: WovenExpr,
        then_branch: WovenStmt,
        else_branch: Option<Box<WovenStmt>>,
    ) -> GenResult<u8> {
        let w_cond = self.gen_from_expr(condition)?;

        let then = self.write_jump(Instruction::JumpIfFalse {
            condition_reg: w_cond,
            offset: 0xffff,
        });

        // generate then block code
        self.gen_from_stmt(then_branch)?;

        if let Some(else_) = else_branch {
            let else_idx = self.write_jump(Instruction::Jump { offset: 0xffff });

            // patch it up, since we got where it ends
            self.patch_jump(then)?;

            // generate else block code
            self.gen_from_stmt(*else_)?;

            self.patch_jump(else_idx)?;
        } else {
            self.patch_jump(then)?;
        }

        Ok(w_cond)
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
                self.instructions.push(Instruction::Negate {
                    dest: dest,
                    r1: register,
                });
                Ok(dest)
            }
            TokenType::Bang => {
                self.instructions.push(Instruction::Not {
                    dest: dest,
                    r1: register,
                });
                Ok(dest)
            }
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
            TRUTH => self.gen_bin_truth_op(r1, r2, op),
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

    // Quick helper function for all binary operation code!
    fn gen_bin_op<F>(&mut self, left: u8, right: u8, inst_builder: F) -> GenResult<u8>
    where
        F: FnOnce(u8, u8, u8) -> Instruction,
    {
        let dest_reg = self.get_next_register()?;
        let less = inst_builder(dest_reg, left, right);
        self.instructions.push(less);
        Ok(dest_reg)
    }

    fn gen_bin_truth_op(&mut self, left: u8, right: u8, op: Token) -> GenResult<u8> {
        match op.token_type {
            TokenType::Greater => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Greater {
                    dest: dest,
                    r1: left,
                    r2: right,
                })?;
            }

            TokenType::Less => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Less {
                    dest,
                    r1,
                    r2,
                })?;
            }

            TokenType::LessEqual => {
                let reg = self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Greater {
                    dest,
                    r1,
                    r2,
                })?;
                self.instructions
                    .push(Instruction::Not { dest: reg, r1: reg });
            }

            TokenType::GreaterEqual => {
                let reg = self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Less {
                    dest,
                    r1,
                    r2,
                })?;
                self.instructions
                    .push(Instruction::Not { dest: reg, r1: reg });
            }

            TokenType::EqualEqual => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Equal {
                    dest,
                    r1,
                    r2,
                })?;
            }

            TokenType::BangEqual => {
                let reg = self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Equal {
                    dest,
                    r1,
                    r2,
                })?;
                self.instructions
                    .push(Instruction::Not { dest: reg, r1: reg });
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

    fn gen_num_op(&mut self, left: u8, right: u8, op: Token) -> GenResult<u8> {
        match op.token_type {
            TokenType::Plus => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Add {
                    dest,
                    r1,
                    r2,
                })?;
            }
            TokenType::Minus => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Subtract {
                    dest,
                    r1,
                    r2,
                })?;
            }
            TokenType::Slash => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Divide {
                    dest,
                    r1,
                    r2,
                })?;
            }
            TokenType::Star => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Multiply {
                    dest,
                    r1,
                    r2,
                })?;
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
