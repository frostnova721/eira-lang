use std::{array::from_fn, collections::HashMap, rc::Rc};

use crate::runtime::{
    operation::OpCode,
    spell::{ClosureObject, SpellObject},
    value::{Value, print_value},
};

pub enum InterpretResult {
    CompileError,
    RuntimeError,
    InterpretOk,
}

struct CallFrame {
    ip: usize,
    closure: Rc<ClosureObject>,
    slot_start: usize,
}

impl CallFrame {
    #[inline(always)]
    pub fn read_byte(&mut self) -> u8 {
        let b = self.closure.spell.bytecode[self.ip];
        self.ip += 1;
        b
    }

    pub fn read_u16(&mut self) -> u16 {
        let (a, b) = (self.read_byte(), self.read_byte());
        u16::from_le_bytes([a, b])
    }

    pub fn read_constant(&mut self) -> &Value {
        let ind = self.read_u16();
        &self.closure.spell.constants[ind as usize]
    }
}

pub struct EiraVM {
    registers: [Value; 256],
    frames: Vec<CallFrame>,

    globals: HashMap<String, Value>,
    stack: Vec<Value>,
}

impl EiraVM {
    pub fn init(byte_code: Vec<u8>, constants: Vec<Value>) -> Self {
        let mut vm = EiraVM {
            registers: from_fn(|_| Value::Emptiness),
            globals: HashMap::new(),
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(256), // initally
        };

        let closure = ClosureObject {
            spell: SpellObject {
                arity: 0,
                bytecode: byte_code,
                constants: constants,
                name: None,
                upvalue_count: 0,
            },
            upvalues: vec![],
        };

        let frame = CallFrame {
            closure: Rc::new(closure),
            ip: 0,
            slot_start: 0,
        };

        vm.frames.push(frame);

        vm
    }

    fn runtime_error(&mut self, msg: &str) {
        println!(
            "Oh no! The VM broke down. \nError: {} at line {}:{}",
            msg, 0, 0
        );

        // ignore other instructions.
        // while self.ip < self.bytecode.len()-1 {
        //     self.ip += 1;
        // }
    }

    fn last_frame(&mut self) -> &CallFrame {
        self.frames.last().expect("frames to be not empty.")
    }

    fn last_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("frames to be not empty.")
    }

    pub fn start(&mut self) -> InterpretResult {
        macro_rules! binary_op {
            ($frame:expr, $op:tt) => {{
                let dest = $frame.read_byte();
                let r1 = $frame.read_byte();
                let r2 = $frame.read_byte();
                let v1 = self.get_register(r1);
                let v2 = self.get_register(r2);

                match (v1, v2) {
                    (Value::Number(n1), Value::Number(n2)) => {
                            self.set_register(dest, Value::from(n1 $op n2));
                        },
                        _ => {
                            self.runtime_error("Operands should be 2 numbers!");
                            return InterpretResult::RuntimeError;
                        }
                }
            }};
        }

        loop {
            // let byte = {
            //     self.last_frame_mut().read_byte()
            // };
            let frame = self.last_frame_mut();
            let op = OpCode::try_from(frame.read_byte()).unwrap();
            // println!("{}", &op.to_debug_string());
            match op {
                OpCode::Add => {
                    let dest = frame.read_byte();
                    let r1 = frame.read_byte();
                    let r2 = frame.read_byte();
                    let v1 = self.get_register(r1);
                    let v2 = self.get_register(r2);
                    match (v1, v2) {
                        (Value::Number(n1), Value::Number(n2)) => {
                            self.set_register(dest, Value::Number(n1 + n2));
                        }
                        (Value::String(s1), Value::String(s2)) => {
                            let new_string = s1.to_string() + s2.as_str();
                            self.set_register(dest, Value::String(new_string.into()));
                        }
                        _ => {
                            self.runtime_error("Operands should be 2 numbers!");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Subtract => {
                    binary_op!(frame, -)
                }
                OpCode::Divide => {
                    binary_op!(frame, /)
                }
                OpCode::Multiply => {
                    binary_op!(frame, *)
                }
                OpCode::Equal => {
                    let dest = frame.read_byte();
                    let r1 = frame.read_byte();
                    let r2 = frame.read_byte();
                    let a = self.get_register(r1);
                    let b = self.get_register(r2);
                    self.set_register(dest, Value::Bool(a.equals(&b)));
                }
                OpCode::Greater => {
                    binary_op!(frame, >)
                }
                OpCode::Less => {
                    binary_op!(frame, <)
                }
                OpCode::False => {
                    let dest = frame.read_byte();
                    self.set_register(dest, Value::Bool(false));
                }
                OpCode::True => {
                    let dest = frame.read_byte();
                    self.set_register(dest, Value::Bool(true));
                }
                OpCode::Negate => {
                    let dest = frame.read_byte();
                    let src_ind = frame.read_byte();
                    let source = self.get_register(src_ind);

                    match source {
                        Value::Number(n) => self.set_register(dest, Value::Number(-n)),
                        _ => {
                            self.runtime_error("What???!! Negation needs a number operand.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Not => {
                    let dest = frame.read_byte();
                    let src_ind = frame.read_byte();
                    let source = self.get_register(src_ind);

                    match source {
                        Value::Bool(b) => self.set_register(dest, Value::Bool(!b)),
                        _ => {
                            self.runtime_error("What???!! Not needs a boolean operand.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Constant => {
                    let dest = frame.read_byte();
                    let val = frame.read_constant().clone();
                    // println!("Loaded {:?}", val);
                    self.set_register(dest, val);
                }
                OpCode::Print => {
                    let i = frame.read_byte();
                    let val = self.get_register(i);
                    print_value(val.clone())
                }
                OpCode::SetGlobal => {
                    // TODO: change from hashmaps to arrays
                    let src_reg_ind = frame.read_byte();
                    let var_name_value = frame.read_constant().clone();
                    let value = self.get_register(src_reg_ind);
                    if let Value::String(name) = var_name_value {
                        self.globals.insert(name.to_string(), value.clone());
                    } else {
                        self.runtime_error(
                            "Fatal: A string was expected for the global variable name.",
                        );
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::GetGlobal => {
                    let dest_reg = frame.read_byte();
                    let val = frame.read_constant().clone();
                    if let Value::String(name) = val {
                        let global = self.globals.get(&name.to_string());
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
                    let src = frame.read_byte();
                    let slot = frame.read_u16() as usize;

                    if slot >= self.stack.len() {
                        self.stack.resize(slot + 1, Value::Emptiness);
                    }

                    let value =
                        std::mem::replace(&mut self.registers[src as usize], Value::Emptiness);
                    self.stack[slot] = value;
                }
                OpCode::GetLocal => {
                    let dest = frame.read_byte();
                    let slot = frame.read_u16() as usize;
                    match &self.stack[slot] {
                        Value::Number(n) => self.registers[dest as usize] = Value::Number(*n),
                        Value::Bool(b) => self.registers[dest as usize] = Value::Bool(*b),
                        Value::Emptiness => self.registers[dest as usize] = Value::Emptiness,
                        _ => {
                            self.registers[dest as usize] = self.stack[slot].clone()
                        }
                    }
                    // let val = self.stack[slot].clone();
                    // self.set_register(dest, val);
                }
                OpCode::Emptiness => {
                    let dest_reg = frame.read_byte();
                    self.set_register(dest_reg, Value::Emptiness);
                }

                OpCode::PopStack => {
                    let mut count = frame.read_u16();
                    while count > 0 {
                        self.stack.pop();
                        count -= 1;
                    }
                }

                OpCode::Jump => {
                    let offset = frame.read_u16();
                    frame.ip += offset as usize;
                }
                OpCode::JumpIfFalse => {
                    let condition_reg = frame.read_byte();
                    let offset = frame.read_u16();

                    if self.get_register(condition_reg).is_falsey() {
                        let frame = self.last_frame_mut();
                        frame.ip += offset as usize;
                    }
                }
                OpCode::Loop => {
                    let offset = frame.read_u16();
                    frame.ip -= offset as usize;
                }
                OpCode::Halt => break,
            }
        }

        return InterpretResult::InterpretOk;
    }

    fn get_register(&self, index: u8) -> &Value {
        &self.registers[index as usize]
    }

    fn set_register(&mut self, index: u8, value: Value) {
        self.registers[index as usize] = value;
    }
}
