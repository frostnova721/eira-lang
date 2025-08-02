
use crate::{
    instruction::Instruction,
    scanner::{Scanner, Token},
    token_type::TokenType,
    value::Value,
};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,

    // chunk: Chunk,
    pub constants: Vec<Value>,

    pub instructions: Vec<Instruction>,

    current_register: u8,

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
        }
    }

    pub fn compile(&mut self) -> Vec<Instruction> {
        self.advance();

        while !self.match_token(TokenType::Eof) {
            self.declaration();
        }

        self.instructions.push(Instruction::Halt);

        self.instructions.clone()
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
        // let ind = self.chunk.add_constant(value);

        // index is encoded in little endian format
        // let reg = self.chunk.get_next_reg();
        // self.write_instruction(
        //     OpCode::Constant,
        //     reg,
        //     (ind & 0x00ff) as u8,
        //     ((ind >> 8) & 0x00ff) as u8,
        // );
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
        (self.constants
            .len() - 1)
            .try_into()
            .expect("Index went out of 16bit limit!")
    }

    pub fn get_next_reg(&mut self) -> u8 {
        if self.current_register == u8::MAX {
            panic!("Maximum registers allocated! Register overflow?!")
        }
        self.current_register += 1;
        self.current_register
    }

    pub fn get_last_allocated_register(&self) -> u8 {
        self.current_register
    }

    // the main guys
    fn declaration(&mut self) {
        if self.match_token(TokenType::Spell) {
            self.spell_declaration();
        } else if self.match_token(TokenType::Mark) {
            self.variable_declaration();
        } else {
            self.statement();
        }

        if self.panic {
            self.sync();
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Chant) {
            self.chant_statement();
        }
        // else if self.match_token(TokenType::)
        else {
            self.expression_statement();
        }
    }

    fn expression(&mut self) -> u8 {
        self.parse_precedence(Precedence::Assign)
    }

    fn grouping(&mut self, _can_assign: bool) -> u8 {
        let exp_res = self.expression();
        self.consume(TokenType::ParenRight, "Close the bracket!\nError: Expected ')' after expression.");
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
        self.write_constant(Value::String(string))
    }

    fn spell_declaration(&mut self) {}

    fn variable_declaration(&mut self) {}

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

    fn variable(&mut self, can_assign: bool) -> u8 {
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
