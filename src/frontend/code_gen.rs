use std::{collections::HashMap, rc::Rc, str, u8};

use crate::{
    assembler::Assembler,
    frontend::{
        expr::WovenExpr,
        reagents::WovenReagent,
        scanner::Token,
        stmt::WovenStmt,
        symbol_table::Symbol,
        tapestry::Tapestry,
        token_type::TokenType,
        weaves::{Weave, Weaves},
    },
    print_instructions,
    runtime::instruction::Instruction,
    values::Value,
    values::spell::{ClosureObject, SpellInfo, SpellObject},
};

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
    pub print_instructions: bool,

    woven_ast: Vec<WovenStmt>,
    instructions: Vec<Instruction>,

    register_index: u8,

    constants: Vec<Vec<Value>>,
    constants_idx_map: Vec<HashMap<Value, u16>>, // Stack of maps, one per constant pool

    loop_blocks: Vec<LoopBlock>,

    in_spell: bool, // track if context is within a spell
    curr_upval_count: usize,
    upval_map: HashMap<(usize, usize), usize>, // map of (depth, slot_idx) to register index
}

impl CodeGen {
    pub fn new(w_ast: Vec<WovenStmt>) -> Self {
        CodeGen {
            woven_ast: w_ast,
            instructions: vec![],
            register_index: 0,
            constants: vec![vec![]],
            constants_idx_map: vec![HashMap::new()], // Initialize with one map for main pool
            loop_blocks: vec![],
            in_spell: false,
            curr_upval_count: 0,
            upval_map: HashMap::new(),
            print_instructions: false,
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
        if self.register_index == 0 {
            return 0;
        }
        self.register_index - 1
    }

    fn add_constant(&mut self, value: Value) -> GenResult<u16> {
        // Check if constant exists in current pool's map
        if let Some(val) = self.constants_idx_map.last().unwrap().get(&value) {
            return Ok(*val);
        }

        // else add the constant to table and return the index
        let ind = self.constants.last().unwrap().len() as u16;
        self.constants.last_mut().unwrap().push(value.clone());
        self.constants_idx_map
            .last_mut()
            .unwrap()
            .insert(value, ind);
        Ok(ind)
    }

    /// Writes a [Constant] instruction to the bytecode.
    /// Returns the register where the constant is stored.
    ///
    /// If the constant already exists, it reuses the existing one.
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
                return Err(error(&format!(
                    "Hmmm... this error shouldnt be thrown! If you are encountering this, congrats! I see a good future in you.Error: Jump patch failed.\
                    \nExpected a 'JUMP' instruction, got {:?}",
                    self.instructions[jump_idx]
                )));
            }
        }

        Ok(())
    }

    fn write_loop(&mut self, start: usize) -> GenResult<usize> {
        let body_bytes_size: usize = self.instructions[start..]
            .iter()
            .map(|inst| inst.len())
            .sum();

        // 2 byte for u16's size, 1 for opcode (2+1=3 iydk)
        let total_offset = body_bytes_size + 3;

        if total_offset > u16::MAX as usize {
            return Err(error("Loop Jump Offset exceeds the 2byte limit."));
        }

        self.instructions.push(Instruction::Loop {
            offset: total_offset as u16,
        });
        Ok(self.instructions.len() - 1)
    }

    fn patch_jump_to(&mut self, jump_idx: usize, target_idx: usize) -> GenResult<()> {
        if jump_idx >= self.instructions.len() || target_idx >= self.instructions.len() {
            return Err(error("Invalid jump patch indices!"));
        }

        // Compute byte distance from the instruction after the jump to the target instruction
        let mut offset = 0usize;
        for i in (jump_idx + 1)..target_idx {
            offset += self.instructions[i].len();
        }

        if offset > u16::MAX as usize {
            return Err(error("Jump offset exceeds 16-bit limit!"));
        }

        match &mut self.instructions[jump_idx] {
            Instruction::Jump { offset: o } => *o = offset as u16,
            Instruction::JumpIfFalse { offset: o, .. } => *o = offset as u16,
            _ => {
                return Err(error(&format!(
                    "Patch target at index {} is not a jump instruction: {:?}",
                    jump_idx, self.instructions[jump_idx]
                )));
            }
        }

        Ok(())
    }

    //--------------- Interface/ Public fns ---------------

    // Thought this name is fun, nothing else, its the main entry point btw
    pub fn summon_bytecode(&mut self) -> GenResult<Vec<u8>> {
        let stmts = self.woven_ast.clone();

        let _ = self.gen_from_stmts(stmts)?;

        self.instructions.push(Instruction::Halt);

        if self.print_instructions {
            print_instructions(
                "<0: The Origin>",
                &self.instructions,
                &self.constants.last().unwrap(),
            );
        }

        let bc = Assembler::convert_to_byte_code(&self.instructions);
        Ok(bc) // change later!
    }

    pub fn get_constants(&mut self) -> Vec<Value> {
        self.constants.last_mut().unwrap().clone()
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
                name: _,
                mutable: _,
                initializer,
                symbol,
            } => self.gen_var_decl_instruction(initializer, symbol),
            WovenStmt::Fate {
                condition,
                then_branch,
                else_branch,
            } => self.gen_fate_instructions(condition, *then_branch, else_branch),
            WovenStmt::While { condition, body } => self.gen_while_instructions(condition, *body),
            WovenStmt::Chant { expression } => self.gen_chant_stmt(expression),
            WovenStmt::Block { statements } => self.gen_from_stmts(statements),
            WovenStmt::Sever { token: _ } => self.gen_sever_instructions(),
            WovenStmt::Flow { token: _ } => self.gen_flow_instructions(),
            WovenStmt::Spell {
                name,
                reagents,
                body,
                spell,
            } => self.gen_spell_instructions(name, reagents, *body, spell),
            WovenStmt::Release { token: _, expr } => self.gen_release_instructions(expr),
            WovenStmt::Sign { name, marks } => todo!(),
        }
    }

    /// Match the type of expr and generate corresponding instruction
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
                tapestry: _,
            } => self.gen_unary_instruction(*operand, operator),
            WovenExpr::Literal {
                value,
                tapestry: _,
                token: _,
            } => {
                let val = self.write_constant(value)?;
                Ok(val)
            }
            WovenExpr::Variable {
                name: _,
                tapestry: _,
                symbol,
            } => self.gen_variable_instruction(symbol),
            WovenExpr::Grouping {
                expression,
                tapestry: _,
            } => self.gen_from_expr(*expression),
            WovenExpr::Assignment {
                name: _,
                value,
                tapestry: _,
                symbol,
            } => self.gen_assignment_instruction(*value, symbol),
            WovenExpr::Cast {
                reagents,
                callee,
                tapestry,
                spell_symbol,
            } => self.gen_cast_instruction(reagents, callee, tapestry, spell_symbol),
        }
    }

    fn gen_cast_instruction(
        &mut self,
        reagents: Vec<WovenExpr>,
        _callee: Token,
        _tapestry: Tapestry,
        spell_symbol: Symbol,
    ) -> GenResult<u8> {
        let spell_reg = self.gen_variable_instruction(spell_symbol)?;
        // Evaluate reagents and capture their result registers in order
        let mut reagent_regs: Vec<u8> = Vec::with_capacity(reagents.len());
        for reagent in reagents.iter() {
            let r = self.gen_from_expr(reagent.clone())?;
            reagent_regs.push(r);
        }

        // self.register_index = reg_idx;

        if reagents.len() > u8::MAX as usize {
            return Err(error(
                "Too many reagents passed to cast! This should'nt be thrown, pls report!",
            ));
        }

        let dest = self.get_next_register()?;

        let reg_start = if reagent_regs.is_empty() {
            self.register_index
        } else if reagent_regs.len() == 1 {
            reagent_regs[0]
        } else {
            let mut contiguous = true;
            for w in reagent_regs.windows(2) {
                if w[1] != w[0].saturating_add(1) {
                    contiguous = false;
                    break;
                }
            }

            if contiguous {
                reagent_regs[0]
            } else {
                // Pack reagents into a fresh contiguous block
                let start = self.register_index; // first one goes here
                for (_, &src) in reagent_regs.iter().enumerate() {
                    let dest = self.get_next_register()?;
                    // dest should be start + i
                    self.instructions.push(Instruction::Move {
                        dest: dest,
                        source: src as u16,
                    });
                }
                start
            }
        };

        self.instructions.push(Instruction::Cast {
            dest,
            spell_reg,
            reg_start,
        });

        Ok(dest)
    }

    fn gen_release_instructions(&mut self, expr: Option<WovenExpr>) -> GenResult<u8> {
        // Generate release value (or Emptiness if none) and emit Release instruction
        let dest = if let Some(e) = expr {
            self.gen_from_expr(e)?
        } else {
            let d = self.get_next_register()?;
            self.instructions.push(Instruction::Emptiness { dest: d });
            d
        };

        self.instructions.push(Instruction::Release { dest });
        Ok(dest)
    }

    fn gen_spell_instructions(
        &mut self,
        name: Token,
        reagents: Vec<WovenReagent>,
        body: WovenStmt,
        spell_info: SpellInfo,
    ) -> GenResult<u8> {
        // Save current state before entering spell compilation context
        let saved_reg_idx = self.register_index;
        let mut spell_instructions = Vec::new();
        let saved_curr_upval_count = self.curr_upval_count;
        let saved_inspell = self.in_spell;
        let saved_upval_map = self.upval_map.clone();

        // Temporarily swap instructions to compile spell body
        std::mem::swap(&mut self.instructions, &mut spell_instructions);

        // state modifications for upvalues management
        let upval_count = spell_info.upvalues.len();
        self.in_spell = true;
        self.curr_upval_count = upval_count;

        for (i, upv) in spell_info.upvalues.iter().enumerate() {
            // Use (depth, index) as key to avoid collisions between upvalues and locals
            self.upval_map.insert((upv.depth, upv.index), i);
        }

        // Push a new constant pool and index map for spell
        self.constants.push(vec![]);
        self.constants_idx_map.push(HashMap::new());
        self.register_index = (upval_count + reagents.len()) as u8; // Reserve registers for reagents

        // Compile the body
        self.gen_from_stmt(body)?;

        // Add implicit return if missing
        let needs_return = !matches!(self.instructions.last(), Some(Instruction::Release { .. }));
        if needs_return {
            let ret_reg = self.get_next_register()?;
            self.instructions
                .push(Instruction::Emptiness { dest: ret_reg });
            self.instructions
                .push(Instruction::Release { dest: ret_reg });
        }

        if self.print_instructions {
            print_instructions(
                &name.lexeme,
                &self.instructions,
                &self.constants.last().unwrap(),
            );
        }

        // Get the compiled results
        let spell_bytecode = Assembler::convert_to_byte_code(&self.instructions);
        let spell_constants = self.constants.pop().unwrap();
        self.constants_idx_map.pop(); // Pop the spell's constant map

        let spell = SpellObject {
            name: Some(name.lexeme.clone()),
            arity: reagents.len() as u8,
            upvalue_count: upval_count as i32,
            constants: spell_constants,
            bytecode: spell_bytecode,
        };
        let closure = ClosureObject {
            spell: Rc::new(spell),
            upvalues: spell_info.upvalues.clone(),
        };

        // Restore the main instructions and register state
        std::mem::swap(&mut self.instructions, &mut spell_instructions);
        self.register_index = saved_reg_idx;
        self.in_spell = saved_inspell;
        self.curr_upval_count = saved_curr_upval_count;
        self.upval_map = saved_upval_map;

        // Write the constant and set the value
        let const_idx = self.write_constant(Value::Closure(Rc::new(closure)))?;
        self.set_value_instruction(spell_info.symbol, const_idx)?;

        Ok(const_idx)
    }

    fn gen_flow_instructions(&mut self) -> GenResult<u8> {
        if self.loop_blocks.is_empty() {
            return Err(error("flow can only be performed inside a loop block!"));
        }
        let ind = self.write_jump(Instruction::Jump { offset: 0xffff });
        self.loop_blocks.last_mut().unwrap().flows.push(ind);

        Ok(self.register_index) // dummy
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

        // get the final index of the loop for the flow to jump
        let loop_idx = self.write_loop(start)?;

        self.patch_jump(exit)?;

        let block = self.loop_blocks.pop().unwrap();
        let severs = block.severs;
        let flows = block.flows;

        for jump in severs {
            self.patch_jump(jump)?;
        }

        for jump in flows {
            self.patch_jump_to(jump, loop_idx)?;
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
        self.set_value_instruction(symbol, reg)?;
        Ok(reg)
    }

    /// Checks the depth, sets as local if depth > 0 else as a global with a value if provided.
    fn gen_var_decl_instruction(
        &mut self,
        initializer: Option<WovenExpr>,
        symbol: Symbol,
    ) -> GenResult<u8> {
        let src = match initializer {
            Some(init) => self.gen_from_expr(init)?,
            None => self.write_constant(Value::Emptiness)?,
        };

        self.set_value_instruction(symbol, src)?;

        Ok(src)
    }

    fn set_value_instruction(&mut self, symbol: Symbol, src_reg: u8) -> GenResult<()> {
        if symbol.depth > 0 {
            // Calculate the target register for this variable
            let target_reg = if self.in_spell {
                // Check if this variable is an upvalue
                if let Some(upv_reg) = self.upval_map.get(&(symbol.depth, symbol.slot_idx)) {
                    // It's an upvalue, use its register directly
                    *upv_reg as u8
                } else {
                    // Inside a spell, locals are offset by upvalue count
                    (self.curr_upval_count + symbol.slot_idx) as u8
                }
            } else {
                symbol.slot_idx as u8
            };

            // If src_reg != target_reg, we need to move the value
            if src_reg != target_reg {
                self.instructions.push(Instruction::Move {
                    dest: target_reg,
                    source: src_reg as u16,
                });
            }
        } else {
            let c_ind = self.add_constant(Value::String(symbol.name.into()))?;
            self.instructions.push(Instruction::SetGlobal {
                src_reg,
                const_index: c_ind,
            });
        }
        Ok(())
    }

    fn gen_variable_instruction(&mut self, symbol: Symbol) -> GenResult<u8> {
        if symbol.depth > 0 {
            if self.in_spell {
                // Check if this variable is an upvalue using (depth, slot_idx) as key
                // This will prevent collision between upvalues and locals with same slot_idx
                if let Some(upv_reg) = self.upval_map.get(&(symbol.depth, symbol.slot_idx)) {
                    // The value is an upvalue! jst return its register
                    return Ok(*upv_reg as u8);
                }

                // Not an upvalue, so it's a local/parameter in current spell
                // Locals are offset by the upvalue count
                return Ok((self.curr_upval_count + symbol.slot_idx) as u8);
            }
            Ok(symbol.slot_idx as u8)
        } else {
            let dest = self.get_next_register()?;
            let const_idx = self.add_constant(Value::String(symbol.name.into()))?;
            self.instructions.push(Instruction::GetGlobal {
                dest: dest,
                const_index: const_idx,
            });
            Ok(dest)
        }
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
            num if num == Weaves::NumWeave.get_weave().tapestry.0 => self.gen_num_op(r1, r2, op),
            truth if truth == Weaves::TruthWeave.get_weave().tapestry.0 => {
                self.gen_bin_truth_op(r1, r2, op)
            }
            text if text == Weaves::TextWeave.get_weave().tapestry.0 => {
                self.gen_bin_text_op(r1, r2, op)
            }
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

    fn gen_bin_text_op(&mut self, left: u8, right: u8, op: Token) -> GenResult<u8> {
        match op.token_type {
            TokenType::Plus => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Concat {
                    dest,
                    r1,
                    r2,
                })?;
            }
            _ => {
                // This error msg should be shown to the user, and... if it does, compiler is bugged
                return Err(error(&format!(
                    "Strand for '{}' operation hasnt been entangled with Eira realms!.\nThis error shouldn't be thrown, Report it to devs!",
                    op.lexeme
                )));
            }
        }
        Ok(self.get_last_allocated_register())
    }

    fn gen_bin_truth_op(&mut self, left: u8, right: u8, op: Token) -> GenResult<u8> {
        match op.token_type {
            TokenType::Greater => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Greater {
                    dest,
                    r1,
                    r2,
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
            TokenType::Percent => {
                self.gen_bin_op(left, right, |dest, r1, r2| Instruction::Mod {
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
            x if x == Weaves::NumWeave.get_weave().tapestry.0 => Ok(Weaves::NumWeave.get_weave()),
            x if x == Weaves::TextWeave.get_weave().tapestry.0 => Ok(Weaves::TextWeave.get_weave()),
            x if x == Weaves::TruthWeave.get_weave().tapestry.0 => {
                Ok(Weaves::TruthWeave.get_weave())
            }
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
