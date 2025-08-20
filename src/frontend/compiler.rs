use std::{collections::HashMap, vec};

use crate::{
    frontend::{scanner::{Scanner, Token}, token_type::TokenType}, runtime::{instruction::{self, Instruction}, value::Value}
};

#[derive(Debug, Clone)]
struct Local {
    name: Token,
    depth: i32,
    mutable: bool,
}

#[derive(Debug, Clone)]
struct Global {
    name: Token,
    mutable: bool,
}

struct LoopBlock {
    severs: Vec<usize>,
    flows: Vec<usize>,
}

pub struct Compiler<'a> {
    scanner: Scanner<'a>,

    // chunk: Chunk,
    pub constants: Vec<Value>,

    pub instructions: Vec<Instruction>,

    current_register: u8,

    scope_depth: i32,
    locals: Vec<Local>,
    globals: HashMap<String, Global>,

    loop_blocks: Vec<LoopBlock>,

    current: Token,
    previous: Token,

    panic: bool,
    error: bool,
}

impl<'a> Compiler<'a> {
    pub fn init_compiler(source: &'a str) -> Self {
        // set up some default placeholder
        let temp_token = Token {
            lexeme: "tempo tokan!".to_string(),
            line: 0,
            token_type: TokenType::Error, // temp
        };

        Compiler {
            current: temp_token.clone(),
            constants: vec![],
            instructions: vec![],
            current_register: 0,
            error: false,
            panic: false,
            previous: temp_token.clone(),
            scanner: Scanner::init(source),
            scope_depth: 0,
            locals: vec![],
            globals: HashMap::new(),
            loop_blocks: vec![],
        }
    }

    pub fn compile(&mut self) -> Result<Vec<Instruction>, &str> {
        self.advance();

        while !self.match_token(TokenType::Eof) {
            self.declaration();
        }

        self.instructions.push(Instruction::Halt);

        if self.error {
            return Err("error 421");
        }

        Ok(self.instructions.to_vec())
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            }
            // error token reporting!
            self.throw_error_at_current(&self.current.lexeme.clone());
        }
    }

    fn consume(&mut self, expect: TokenType, msg: &str) {
        if self.current.token_type == expect {
            self.advance();
            return;
        }

        self.throw_error_at_current(msg);
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        token_type == self.current.token_type
    }

    fn error_at(&mut self, msg: &str, pos: Token) {
        if self.panic {
            return;
        }
        self.panic = true;
        println!(
            "Woah! Caught an incorrect magic at line: {}:{}\nError: {}",
            pos.line, self.scanner.current, msg
        );
        self.error = true;
    }

    fn throw_error_at_current(&mut self, msg: &str) {
        self.error_at(msg, self.current.clone());
    }

    fn throw_error(&mut self, msg: &str) {
        self.error_at(msg, self.previous.clone());
    }

    fn sync(&mut self) {
        self.panic = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::SemiColon {
                return;
            }

            match self.current.token_type {
                TokenType::Tome => return,
                TokenType::Spell => return,
                TokenType::Mark => return,
                TokenType::Bind => return,
                TokenType::Seal => return,
                TokenType::While => return,
                TokenType::Chant => return,
                TokenType::Release => return,
                TokenType::Fate => return,

                _ => {}
            }

            self.advance();
        }
    }

    fn write_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    fn write_constant(&mut self, value: Value) -> u8 {
        let reg = self.get_next_reg();
        let ind = self.add_constant(value);
        self.write_instruction(Instruction::Constant {
            dest: reg,
            const_index: ind,
        });
        reg
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        if self.constants.len() >= u16::MAX as usize {
            panic!("Thats a lot of constants! Max amount of constants has been reached.")
        }
        self.constants.push(value);
        (self.constants.len() - 1)
            .try_into()
            .expect("Index went out of 16bit limit!")
    }

    pub fn get_next_reg(&mut self) -> u8 {
        if self.current_register == u8::MAX {
            panic!("Maximum registers allocated! Register overflow?!")
        }
        self.current_register += 1;
        self.current_register - 1
    }

    pub fn get_last_allocated_register(&self) -> u8 {
        self.current_register - 1
    }

    // the main guys
    fn declaration(&mut self) {
        if self.match_token(TokenType::Spell) {
            self.spell_declaration();
        } else if self.match_token(TokenType::Mark) {
            self.variable_declaration(true);
        } else if self.match_token(TokenType::Bind) {
            self.variable_declaration(false);
        } else {
            self.statement();
        }

        if self.panic {
            self.sync();
        }
    }

    fn start_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        let mut pop_count = 0;

        while self.locals.len() > 0 && self.locals[self.locals.len() - 1].depth > self.scope_depth {
            pop_count += 1;
            self.locals.pop();
        }

        self.write_instruction(Instruction::PopStack {
            pop_count: pop_count,
        });
    }

    fn block(&mut self) {
        while !self.check(TokenType::BraceRight) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(
            TokenType::BraceRight,
            "Wheres the '}'??!\nExpected '}' at the end of a block.",
        );
    }

    fn write_loop(&mut self, start_offset: usize) {
        let body_byte_size: usize = self.instructions[start_offset..]
            .iter()
            .map(|instr| instr.len())
            .sum();

        // Add the size of the Loop instruction (3byte).
        let total_offset = body_byte_size + 3;
        self.write_instruction(Instruction::Loop {
            offset: total_offset as u16,
        });
    }

    fn write_jump(&mut self, instruction: Instruction) -> usize {
        self.write_instruction(instruction);
        self.instructions.len() - 1
    }

    fn patch_jump(&mut self, jump_index: usize) {
        let mut offset = 0;

        for i in (jump_index + 1)..self.instructions.len() {
            offset += self.instructions[i].len();
        }

        if offset > u16::MAX as usize {
            self.throw_error("The magic is too complex(long) to jump over!");
        }

        match &mut self.instructions[jump_index] {
            Instruction::JumpIfFalse { offset: o, ..} => *o = offset as u16,
            Instruction::Jump { offset: o} => *o = offset as u16,
            _ => self.throw_error("Hmmm... this error shouldnt be thrown! If you are encountering this, congrats! I see a good future in you."),
        }
    }

    fn fate_statement(&mut self) {
        let condition_reg = self.expression();
        let then = self.write_jump(Instruction::JumpIfFalse {
            condition_reg: condition_reg,
            offset: 0xffff,
        });

        self.consume(TokenType::BraceLeft, "Expected '{' at start of fate block.");

        self.handle_block();

        if self.match_token(TokenType::Divert) {
            let else_idx = self.write_jump(Instruction::Jump { offset: 0xffff });

            // patch the then part since we know where the else ends!
            self.patch_jump(then);

            self.consume(
                TokenType::BraceLeft,
                "Expected a '{' at start of divert block. Forgot to add it?",
            );
            self.handle_block();
            self.patch_jump(else_idx);
        } else {
            self.patch_jump(then);
        }
    }

    fn loop_statement(&mut self) {
        let start = self.instructions.len();
        let condition_reg = self.expression();

        let exit = self.write_jump(Instruction::JumpIfFalse {
            condition_reg: condition_reg,
            offset: 0xffff,
        });

        self.consume(TokenType::BraceLeft, "Expected '{' at start of loop block.");
        self.loop_blocks.push(LoopBlock {
            severs: vec![],
            flows: vec![],
        });

        self.handle_block();

        self.write_loop(start);

        self.patch_jump(exit);

        let severs = self.loop_blocks.pop().unwrap().severs;

        for jump in severs {
            self.patch_jump(jump);
        }
    }

    // creates a scope, runs the block code it and clears it up
    fn handle_block(&mut self) {
        self.start_scope();
        self.block();
        self.end_scope();
    }

    fn sever_statement(&mut self) {
        if self.loop_blocks.is_empty() {
            return self.throw_error("Only the loops can be severed.");
        }
        self.consume(TokenType::SemiColon, "Expected ';' after 'sever'.");
        let ind = self.write_jump(Instruction::Jump { offset: 0xffff });
        self.loop_blocks.last_mut().unwrap().severs.push(ind);
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Chant) {
            self.chant_statement();
        } else if self.match_token(TokenType::BraceLeft) {
            self.start_scope();
            self.block();
            self.end_scope();
        } else if self.match_token(TokenType::Fate) {
            self.fate_statement();
        } else if self.match_token(TokenType::While) {
            self.loop_statement();
        } else if self.match_token(TokenType::Sever) {
            self.sever_statement();
        } else {
            self.expression_statement();
        }
    }

    fn expression(&mut self) -> u8 {
        self.parse_precedence(Precedence::Assign)
    }

    fn grouping(&mut self, _can_assign: bool) -> u8 {
        let exp_res = self.expression();
        self.consume(
            TokenType::ParenRight,
            "Close the bracket!\nError: Expected ')' after expression.",
        );
        exp_res
    }

    fn number(&mut self, _can_assign: bool) -> u8 {
        let val: f64 = self.previous.lexeme.parse().unwrap();
        self.write_constant(Value::Number(val))
    }

    fn literal(&mut self, _can_assign: bool) -> u8 {
        let reg = self.get_next_reg();
        match self.previous.token_type {
            TokenType::True => self.write_instruction(Instruction::True { dest: reg }),
            TokenType::False => self.write_instruction(Instruction::False { dest: reg }),
            _ => {}
        };
        return reg;
    }

    fn string(&mut self, _can_assign: bool) -> u8 {
        let string = self.previous.lexeme.clone();

        // let mut substrings: String;

        // interpolate
        // for (i, c) in string.chars().enumerate() {

        // }
        self.write_constant(Value::String(string.into()))
    }

    fn spell_declaration(&mut self) {}

    fn resolve_local(&mut self, name: &Token) -> Option<u16> {
        for i in (0..self.locals.len()).rev() {
            let local = &self.locals[i];
            if self.identifiers_equal(&local.name, name) {
                if local.depth == -1 {
                    self.throw_error("Cannot read a local mark in its own initializer.");
                }
                return Some(i as u16);
            }
        }
        None
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) -> u8 {
        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            let val_reg = self.get_last_allocated_register();

            if let Some(slot_idx) = self.resolve_local(name) {
                // restrict assigning to bind values
                if !self.locals[slot_idx as usize].mutable {
                    self.throw_error(&format!(
                        "Rebinding the value of '{}' is forbidden!",
                        name.lexeme
                    ))
                }

                self.write_instruction(Instruction::SetLocal {
                    src_reg: val_reg,
                    slot_idx: slot_idx,
                });
            } else {
                if !self.globals.contains_key(&name.lexeme) {
                    self.throw_error(&format!(
                        "The global variable '{}' was not declared!",
                        name.lexeme
                    ));
                }

                let variable = self.globals.get(&name.lexeme).unwrap();
                if !variable.mutable {
                    self.throw_error(&format!(
                        "Rebinding the value of '{}' is forbidden!",
                        name.lexeme
                    ))
                }
                let const_idx = self.identifier_constant(name.clone());
                self.write_instruction(Instruction::SetGlobal {
                    src_reg: val_reg,
                    const_index: const_idx,
                });
            }

            val_reg
        } else {
            let dest = self.get_next_reg();

            if let Some(slot_idx) = self.resolve_local(name) {
                self.write_instruction(Instruction::GetLocal {
                    dest: dest,
                    slot_index: slot_idx,
                });
            } else {
                let name_idx = self.identifier_constant(name.clone());
                self.write_instruction(Instruction::GetGlobal {
                    dest: dest,
                    const_index: name_idx,
                });
            }
            dest
        }
    }

    fn variable(&mut self, can_assign: bool) -> u8 {
        let var_name = &self.previous.clone();
        self.named_variable(var_name, can_assign)
    }

    fn identifier_constant(&mut self, name: Token) -> u16 {
        self.add_constant(Value::String(name.lexeme.into()))
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        a.lexeme == b.lexeme
    }

    fn declare_local(&mut self, name: &Token, mutable: bool) {
        let mut duplicate_found = false;
        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }
            if self.identifiers_equal(name, &local.name) {
                duplicate_found = true;
                break;
            }
        }
        if duplicate_found {
            self.throw_error("A mark with same name already exists in the current realm!");
        }

        // make local with uninitialized state
        self.locals.push(Local {
            depth: -1, // -1 = undefined
            name: name.clone(),
            mutable: mutable,
        });
    }

    fn variable_declaration(&mut self, mutable: bool) {
        // get var name
        self.consume(TokenType::Identifier, "Expected a mark name!");
        let name = self.previous.clone();
        if self.scope_depth > 0 {
            self.declare_local(&name, mutable);
        }

        // self.identifier_constant(self.previous.clone())
        // parse initialiser
        let val_reg = if self.match_token(TokenType::Equal) {
            self.expression()
        } else {
            if !mutable {
                self.throw_error("bind values must be initialized.")
            }
            let reg = self.get_next_reg();
            self.write_instruction(Instruction::Emptiness { dest: reg });
            reg
        };

        self.consume(
            TokenType::SemiColon,
            "Missing ';' after the variable marking!",
        );

        // define the variable
        if self.scope_depth > 0 {
            self.mark_local_initialized();
            let slot = (self.locals.len() - 1) as u16;
            self.write_instruction(Instruction::SetLocal {
                src_reg: val_reg,
                slot_idx: slot,
            });
        } else {
            let lex = name.lexeme.clone();
            if self.globals.contains_key(&lex) {
                self.throw_error("A global mark with the same name is already sealed!")
            }

            let glob = Global {
                mutable: mutable,
                name: name.clone(),
            };

            self.globals.insert(lex, glob);

            let name_ind = self.identifier_constant(name);
            self.write_instruction(Instruction::SetGlobal {
                src_reg: val_reg,
                const_index: name_ind,
            })
        }
    }

    fn mark_local_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let last = self.locals.len() - 1;
        self.locals[last].depth = self.scope_depth;
    }

    fn chant_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Missing ';' after the magic chants!");
        self.write_instruction(Instruction::Print {
            r1: self.get_last_allocated_register(),
        });
    }

    fn binary(&mut self, _can_assign: bool, lhs_reg: u8) -> u8 {
        let op = self.previous.token_type;

        let rule = self.get_rule(op);
        let rhs_reg = self.parse_precedence(rule.precedence.next());

        let next_reg = self.get_next_reg();

        match op {
            TokenType::Plus => self.write_instruction(Instruction::Add {
                dest: next_reg,
                r1: lhs_reg,
                r2: rhs_reg,
            }),
            TokenType::Minus => self.write_instruction(Instruction::Subtract {
                dest: next_reg,
                r1: lhs_reg,
                r2: rhs_reg,
            }),
            TokenType::Star => self.write_instruction(Instruction::Multiply {
                dest: next_reg,
                r1: lhs_reg,
                r2: rhs_reg,
            }),
            TokenType::Slash => self.write_instruction(Instruction::Divide {
                dest: next_reg,
                r1: lhs_reg,
                r2: rhs_reg,
            }),

            TokenType::BangEqual => {
                self.write_instruction(Instruction::Equal {
                    dest: next_reg,
                    r1: lhs_reg,
                    r2: rhs_reg,
                });
                self.write_instruction(Instruction::Not {
                    dest: next_reg,
                    r1: next_reg,
                });
            }

            TokenType::EqualEqual => self.write_instruction(Instruction::Equal {
                dest: next_reg,
                r1: lhs_reg,
                r2: rhs_reg,
            }),

            TokenType::Greater => self.write_instruction(Instruction::Greater {
                dest: next_reg,
                r1: lhs_reg,
                r2: rhs_reg,
            }),
            TokenType::GreaterEqual => {
                self.write_instruction(Instruction::Less {
                    dest: next_reg,
                    r1: lhs_reg,
                    r2: rhs_reg,
                });
                self.write_instruction(Instruction::Not {
                    dest: next_reg,
                    r1: next_reg,
                });
            }

            TokenType::Less => self.write_instruction(Instruction::Less {
                dest: next_reg,
                r1: lhs_reg,
                r2: rhs_reg,
            }),
            TokenType::LessEqual => {
                self.write_instruction(Instruction::Greater {
                    dest: next_reg,
                    r1: lhs_reg,
                    r2: rhs_reg,
                });
                self.write_instruction(Instruction::Not {
                    dest: next_reg,
                    r1: next_reg,
                });
            }
            _ => { /*the customer is unreachable :( */ }
        };

        return next_reg;
    }

    fn unary(&mut self, _can_assign: bool) -> u8 {
        let op_type = self.previous.token_type;
        self.parse_precedence(Precedence::Unary);
        let source = self.get_last_allocated_register();
        let dest = self.get_next_reg();

        match op_type {
            TokenType::Minus => self.write_instruction(Instruction::Negate {
                dest: dest,
                r1: source,
            }),
            TokenType::Bang => self.write_instruction(Instruction::Not {
                dest: dest,
                r1: source,
            }),
            _ => {}
        };
        return dest;
    }

    fn call(&mut self, _can_assign: bool, lhs_reg: u8) -> u8 {
        0
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(
            TokenType::SemiColon,
            "Expected a ';' after the expression. Forgot to add it?",
        );
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> u8 {
        self.advance();
        let rule = self.get_rule(self.previous.token_type).prefix;

        match rule {
            None => {
                self.throw_error("An expression was expected!");
                return 0; // dummy stuff, might change later!
            }
            Some(prefix_rule) => {
                let can_assign = precedence.power() <= Precedence::Assign.power();
                let mut lhs_reg = prefix_rule(self, can_assign);

                while precedence.power()
                    <= self.get_rule(self.current.token_type).precedence.power()
                {
                    self.advance();
                    let infix_rule = self.get_rule(self.previous.token_type).infix.unwrap();
                    lhs_reg = infix_rule(self, can_assign, lhs_reg);
                }

                if can_assign && self.match_token(TokenType::Equal) {
                    self.throw_error("Assignment target provided is invalid! Take a look at it!");
                    return 0;
                }

                return lhs_reg;
            }
        }
    }

    fn get_rule(&self, token_type: TokenType) -> ParseRule<'a> {
        match token_type {
            TokenType::ParenLeft => ParseRule {
                prefix: Some(Self::grouping),
                infix: Some(Self::call),
                precedence: Precedence::Call,
            },
            TokenType::ParenRight => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BraceLeft => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BraceRight => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Dot => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::Call,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Self::unary),
                infix: Some(Self::binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Term,
            },
            TokenType::SemiColon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Self::unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Self::binary),
                precedence: Precedence::Compare,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(Self::variable),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::String => ParseRule {
                prefix: Some(Self::string),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Self::number),
                infix: None,
                precedence: Precedence::None,
            },
            // TokenType::And => ParseRule { prefix: None, infix: Some(Self::and_), precedence: Precedence::None },
            TokenType::Tome => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Flow => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Alias => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Sever => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Divert => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::False => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            // TokenType::For => ParseRule { prefix: None, infix: None, precedence: Precedence::None },
            TokenType::Spell => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Fate => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            // TokenType::Or => ParseRule { prefix: None, infix: Some(Self::or_), precedence: Precedence::Or },
            TokenType::Chant => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Release => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            // TokenType::Origin => ParseRule { prefix: Some(Self::super_), infix: None, precedence: Precedence::None },
            // TokenType::_Self => ParseRule { prefix: Some(Self::this_), infix: None, precedence: Precedence::None },
            TokenType::True => ParseRule {
                prefix: Some(Self::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Mark => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Bind => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            _ => panic!("Some rule went haywire!"),
        }
    }
}

type ParseFun<'a> = fn(&mut Compiler<'a>, bool) -> u8;

type InfixParseFun<'a> = fn(&mut Compiler<'a>, bool, u8) -> u8;

struct ParseRule<'a> {
    prefix: Option<ParseFun<'a>>,
    infix: Option<InfixParseFun<'a>>,
    precedence: Precedence,
}

enum Precedence {
    None,
    Assign,
    Or,
    And,
    Equality,
    Compare,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn next(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assign,
            Precedence::Assign => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Compare,
            Precedence::Compare => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }

    pub fn power(&self) -> u8 {
        match self {
            Precedence::None => 0,
            Precedence::Assign => 1,
            Precedence::Or => 2,
            Precedence::And => 3,
            Precedence::Equality => 4,
            Precedence::Compare => 5,
            Precedence::Term => 6,
            Precedence::Factor => 7,
            Precedence::Unary => 8,
            Precedence::Call => 9,
            Precedence::Primary => 10,
        }
    }

    // pub fn from_power(power: u8) -> Self {
    //     match power {
    //         0 => Precedence::None,
    //         1 => Precedence::Assign,
    //         2 => Precedence::Or,
    //         3 => Precedence::And,
    //         4 => Precedence::Equality,
    //         5 => Precedence::Compare,
    //         6 => Precedence::Term,
    //         7 => Precedence::Factor,
    //         8 => Precedence::Unary,
    //         9 => Precedence::Call,
    //         10 => Precedence::Primary,
    //         _ => panic!("Unknown precedence to match the power!"),
    //     }
    // }
}
