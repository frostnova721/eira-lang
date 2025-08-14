use std::{array::from_fn, collections::HashMap, str::Bytes};

use crate::{
    operation::OpCode,
    value::{self, Value, print_value},
};

pub enum InterpretResult {
    CompileError,
    RuntimeError,
    InterpretOk,
}

pub struct EiraVM {
    bytecode: Vec<u8>,
    registers: [Value; 256],
    constants: Vec<Value>,
    ip: usize,
    stack: Vec<Value>,

    globals: HashMap<String, Value>,
}

impl EiraVM {
    pub fn init(byte_code: Vec<u8>, constants: Vec<Value>) -> Self {
        EiraVM {
            bytecode: byte_code,
            constants: constants,
            registers: from_fn(|_| Value::Emptiness),
            ip: 0,
            globals: HashMap::new(),
            stack: vec![],
        }
    }

    fn runtime_error(&mut self, msg: &str) {
        println!(
            "Oh no! The VM broke down. \nError: {} at line {}:{}",
            msg, 0, 0
        );

        // ignore other instructions.
        while self.ip < self.bytecode.len()-1 {
            self.ip += 1;
        }
    }

    pub fn start(&mut self) -> InterpretResult {
        macro_rules! read_byte {
            () => {{
                let byte = self.bytecode[self.ip];
                self.ip += 1;
                byte
            }};
        }

        macro_rules! read_constant {
            () => {{
                let b1 = read_byte!();
                let b2 = read_byte!();
                &self.constants[u16::from_le_bytes([b1, b2]) as usize]
            }};
        }

        macro_rules! binary_op {
            ($op:tt, $val_type:path) => {{
                let dest = read_byte!();
                let r1 = read_byte!();
                let r2 = read_byte!();
                let v1 = self.get_register(r1).clone();
                let v2 = self.get_register(r2).clone();

                match (v1, v2) {
                    (Value::Number(n1), Value::Number(n2)) => {
                            self.set_register(dest, $val_type(n1 $op n2));
                        },
                        _ => {
                            self.runtime_error("Operands should be 2 numbers!");
                            return InterpretResult::RuntimeError;
                        }
                }
            }};
        }

        macro_rules! read_u16 {
            () => {{
                let a = read_byte!();
                let b = read_byte!();
                u16::from_le_bytes([a, b])
            }};
        }

        loop {
            let op = OpCode::try_from(read_byte!()).unwrap();
            // println!("{}", &op.to_debug_string());
            match op {
                OpCode::Add => {
                    let dest = read_byte!();
                    let r1 = read_byte!();
                    let r2 = read_byte!();
                    let v1 = self.get_register(r1).clone();
                    let v2 = self.get_register(r2).clone();
                    match (v1, v2) {
                        (Value::Number(n1), Value::Number(n2)) => {
                            self.set_register(dest, Value::Number(n1 + n2));
                        }
                        (Value::String(s1), Value::String(s2)) => {
                            let mut new_string= String::new();
                            new_string.push_str(s1.as_str());
                            new_string.push_str(s2.as_str());
                            self.set_register(dest, Value::String(new_string));
                        }
                        _ => {
                            self.runtime_error("Operands should be 2 numbers!");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Subtract => binary_op!(-, Value::Number),
                OpCode::Divide => binary_op!(/, Value::Number),
                OpCode::Multiply => binary_op!(*, Value::Number),
                OpCode::Equal => {
                    let dest = read_byte!();
                    let r1 = read_byte!();
                    let r2 = read_byte!();
                    let a = self.get_register(r1).clone();
                    let b = self.get_register(r2).clone();
                    self.set_register(dest, Value::Bool(a.equals(&b)));
                }
                OpCode::Greater => binary_op!(>, Value::Bool),
                OpCode::Less => binary_op!(<, Value::Bool),
                OpCode::False => { 
                    let dest = read_byte!();
                    self.set_register(dest, Value::Bool(false));
                }
                OpCode::True => {
                     let dest = read_byte!();
                    self.set_register(dest, Value::Bool(true));
                }
                OpCode::Negate => {
                    let dest = read_byte!();
                    let src_ind = read_byte!();
                    let source = self.get_register(src_ind).clone();

                    match source {
                        Value::Number(n) => self.set_register(dest, Value::Number(-n)),
                        _ => {
                            self.runtime_error("What???!! Negation needs a number operand.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Not => {
                    let dest = read_byte!();
                    let src_ind = read_byte!();
                    let source = self.get_register(src_ind).clone();

                    match source {
                        Value::Bool(b) => self.set_register(dest, Value::Bool(!b)),
                        _ => {
                            self.runtime_error("What???!! Not needs a boolean operand.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Constant => {
                    let dest = read_byte!();
                    let val = read_constant!().clone();
                    // println!("Loaded {:?}", val);
                    self.set_register(dest, val);
                }
                OpCode::Print => {
                    let i = read_byte!();
                    let val = self.get_register(i);
                    print_value(val.clone())
                }
                OpCode::SetGlobal => {
                    let src_reg_ind = read_byte!();
                    let value = self.get_register(src_reg_ind).clone();
                    let var_name_value = read_constant!();
                    if let Value::String(name) = var_name_value {
                        self.globals.insert(name.clone(), value);
                    } else {
                        self.runtime_error(
                            "Fatal: A string was expected for the global variable name.",
                        );
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::GetGlobal => {
                    let dest_reg = read_byte!();
                    let val = read_constant!();
                    if let Value::String(name) = val {
                        let global = self.globals.get(name);
                        if let Some(value) = global {
                            self.set_register(dest_reg, value.clone());
                        } else {
                            self.runtime_error(&format!("The mark '{}' was undefined", name));
                        }
                    } else {
                        self.runtime_error(
                            "Fatal: A string was expected for the global variable name.",
                        );
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::SetLocal => {
                    let src = read_byte!();
                    let value = self.get_register(src).clone();
                    let slot = read_u16!() as usize;
                    if slot < self.stack.len() {
                        self.stack[slot] = value;
                    } else {
                        // fill till the slot is reached
                        while self.stack.len() <= slot {
                            self.stack.push(Value::Emptiness);
                        }
                        self.stack[slot] = value;
                    }
                }
                OpCode::GetLocal => {
                    let dest = read_byte!();
                    let slot = read_u16!() as usize;
                    let val = self.stack[slot].clone();
                    // if let Value::String(name) = val {
                    //     // let local =
                    //     // self.runtime_error(&format!("The mark '{}' was undefined.", val));
                    // }
                    self.set_register(dest, val);
                }
                OpCode::Emptiness => {
                    let dest_reg = read_byte!();
                    self.set_register(dest_reg, Value::Emptiness);
                }

                OpCode::PopStack => {
                    let mut count = read_u16!();
                    while count > 0 {
                        self.stack.pop();
                        count -= 1;
                    }
                }

                OpCode::Jump => {
                    let offset = read_u16!();
                    self.ip += offset as usize;
                },
                OpCode::JumpIfFalse => {
                    let condition_reg = read_byte!();
                    let offset = read_u16!();
                    if self.get_register(condition_reg).is_falsey() {
                        self.ip += offset as usize;
                    }
                },
                OpCode::Loop => {
                    let offset = read_u16!();
                    self.ip -= offset as usize;
                },
                OpCode::Halt => break,
            }
        }

        return InterpretResult::InterpretOk;
    }

    fn get_register(&mut self, index: u8) -> &Value {
        &self.registers[index as usize]
    }

    fn set_register(&mut self, index: u8, value: Value) {
        self.registers[index as usize] = value;
    }
}
