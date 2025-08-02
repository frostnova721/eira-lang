use std::array::from_fn;

use crate::{
    operation::OpCode,
    value::{Value, print_value},
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
}

impl EiraVM {
    pub fn init(byte_code: Vec<u8>, constants: Vec<Value>) -> Self {
        EiraVM {
            bytecode: byte_code,
            constants: constants,
            registers: from_fn(|_| Value::Emptiness),
            ip: 0,
        }
    }

    fn runtime_error(&self, msg: &str) {
        println!("Oh no! The VM broke down. \nError: {} at line {}:{}", msg, 0, 0);
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
                    let a =  self.get_register(r1).clone();
                    let b = self.get_register(r2).clone();
                    self.set_register(dest, Value::Bool(a.equals(&b)));
                },
                OpCode::Greater => binary_op!(>, Value::Bool),
                OpCode::Less => binary_op!(<, Value::Bool),
                OpCode::False => {}
                OpCode::True => {}
                OpCode::Negate => {
                    let dest = read_byte!();
                    let src_ind = read_byte!();
                    let source = self.get_register(src_ind).clone();

                    match source {
                     Value::Number(n) =>  self.set_register(dest, Value::Number(-n)),
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
                     Value::Bool(b) =>  self.set_register(dest, Value::Bool(!b)),
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
