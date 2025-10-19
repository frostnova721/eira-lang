use std::{array::from_fn, collections::HashMap, rc::Rc};

use crate::{
    runtime::{
        operation::OpCode,
        spell::{ClosureObject, SpellObject},
    },
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
    return_reg: u8,
}

impl CallFrame {
    #[inline(always)]
    pub fn read_byte(&mut self) -> u8 {
        let b = self.closure.spell.bytecode[self.ip];
        self.ip += 1;
        b
    }

    #[inline(always)]
    pub fn read_three_bytes(&mut self) -> (u8, u8, u8) {
        let ip = self.ip;
        let b1 = self.closure.spell.bytecode[ip];
        let b2 = self.closure.spell.bytecode[ip + 1];
        let b3 = self.closure.spell.bytecode[ip + 2];
        self.ip += 3;
        (b1, b2, b3)
    }

    #[inline(always)]
    pub fn read_u16(&mut self) -> u16 {
        let a = self.closure.spell.bytecode[self.ip];
        let b = self.closure.spell.bytecode[self.ip + 1];
        self.ip += 2;
        u16::from_le_bytes([a, b])
    }

    #[inline(always)]
    pub fn read_constant(&mut self) -> &Value {
        let ind = self.read_u16();
        &self.closure.spell.constants[ind as usize]
    }
}

pub struct EiraVM {
    frames: Vec<CallFrame>,

    globals: HashMap<String, Value>,
    stack: Vec<Value>,
}

impl EiraVM {
    pub fn init(byte_code: Vec<u8>, constants: Vec<Value>) -> Self {
        let mut vm = EiraVM {
            globals: HashMap::new(),
            stack: Vec::with_capacity(256 * 64),
            frames: Vec::with_capacity(256), // initally
        };

        vm.stack.resize(256, Value::Emptiness);

        let closure = ClosureObject {
            spell: Rc::new(SpellObject {
                arity: 0,
                bytecode: byte_code,
                constants: constants,
                name: None,
                upvalue_count: 0,
            }),
            upvalues: vec![],
        };

        let frame = CallFrame {
            closure: Rc::new(closure),
            ip: 0,
            slot_start: 0,
            return_reg: 0,
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

    // fn cast_a_spell(&mut self, spell: &SpellObject) -> Value {
    //     let frame = self.frames.last_mut().unwrap();

    // }

    pub fn start(&mut self) -> InterpretResult {
        macro_rules! binary_op {
            ($frame:expr, $op:tt) => {{
                let (dest, r1, r2) = $frame.read_three_bytes();
                let v1 = get_register!(r1);
                let v2 = get_register!(r2);

                match (v1, v2) {
                    (Value::Number(n1), Value::Number(n2)) => {
                            set_register!(dest, Value::from(n1 $op n2));
                        },
                        _ => {
                            self.runtime_error("Operands should be 2 numbers!");
                            return InterpretResult::RuntimeError;
                        }
                }
            }};
        }

        macro_rules! set_register {
            ($index: expr, $value: expr) => {
                self.stack[$index as usize] = $value
            };
        }

        macro_rules! get_register {
            ($index: expr) => {
                &self.stack[$index as usize]
            };
        }

        loop {
            // let byte = {
            //     self.last_frame_mut().read_byte()
            // };
            let frame = self.frames.last_mut().unwrap();
            let op = OpCode::try_from(frame.read_byte()).unwrap();
            let slot_start = frame.slot_start as usize;

            // println!("{}", &op.to_debug_string());
            match op {
                OpCode::Add => {
                    let (dest, r1, r2) = frame.read_three_bytes();
                    let v1 = get_register!(slot_start + r1 as usize);
                    let v2 = get_register!(slot_start + r2 as usize);
                    // match (v1, v2) {
                    // (Value::Number(n1), Value::Number(n2)) => {
                    set_register!(
                        slot_start + dest as usize,
                        Value::Number(v1.extract_number().unwrap() + v2.extract_number().unwrap())
                    );
                    // }
                    // (Value::String(s1), Value::String(s2)) => {
                    //     let new_string = s1.to_string() + s2.as_str();
                    //     set_register!(dest, Value::String(new_string.into()));
                    // }
                    // _ => {
                    //     self.runtime_error("Operands should be 2 numbers!");
                    //     return InterpretResult::RuntimeError;
                    // }
                    // }
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
                OpCode::Concat => {
                    let (dest, r1, r2) = frame.read_three_bytes();
                    let v1 = get_register!(slot_start + r1 as usize);
                    let v2 = get_register!(slot_start + r2 as usize);
                    set_register!(
                        slot_start + dest as usize,
                        Value::String(Rc::new(
                            v1.extract_string().unwrap() + &v2.extract_string().unwrap()
                        ))
                    );
                }
                OpCode::Equal => {
                    let (dest, r1, r2) = frame.read_three_bytes();
                    let a = get_register!(slot_start + r1 as usize);
                    let b = get_register!(slot_start + r2 as usize);
                    set_register!(slot_start + dest as usize, Value::Bool(a.equals(&b)));
                }
                OpCode::Greater => {
                    binary_op!(frame, >)
                }
                OpCode::Less => {
                    binary_op!(frame, <)
                }
                OpCode::False => {
                    let dest = frame.read_byte();
                    set_register!(slot_start + dest as usize, Value::Bool(false));
                }
                OpCode::True => {
                    let dest = frame.read_byte();
                    set_register!(slot_start + dest as usize, Value::Bool(true));
                }
                OpCode::Negate => {
                    let dest = frame.read_byte();
                    let src_ind = frame.read_byte();
                    let source = get_register!(slot_start + src_ind as usize);

                    match source {
                        Value::Number(n) => {
                            set_register!(slot_start + dest as usize, Value::Number(-n))
                        }
                        _ => {
                            self.runtime_error("What???!! Negation needs a number operand.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Not => {
                    let dest = frame.read_byte();
                    let src_ind = frame.read_byte();
                    let source = get_register!(slot_start + src_ind as usize);

                    match source {
                        Value::Bool(b) => {
                            set_register!(slot_start + dest as usize, Value::Bool(!b))
                        }
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
                    set_register!(slot_start + dest as usize, val);
                }
                OpCode::Print => {
                    let i = frame.read_byte();
                    let val = get_register!(slot_start + i as usize);
                    print_value(val.clone())
                }
                OpCode::SetGlobal => {
                    // TODO: change from hashmaps to arrays
                    let src_reg_ind = frame.read_byte();
                    let var_name_value = frame.read_constant().clone();
                    let value = get_register!(slot_start + src_reg_ind as usize);
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
                            set_register!(slot_start + dest_reg as usize, value.clone());
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
                    let slot = slot_start + frame.read_u16() as usize;

                    if slot >= self.stack.len() {
                        self.stack.resize(slot + 64, Value::Emptiness);
                    }

                    let value = std::mem::replace(
                        &mut self.stack[slot_start + src as usize],
                        Value::Emptiness,
                    );
                    self.stack[slot] = value;
                }
                OpCode::GetLocal => {
                    let dest = frame.read_byte();
                    let slot = frame.read_u16() as usize;
                    self.stack[slot_start + dest as usize] =
                        self.stack[slot_start + slot as usize].clone();
                    // match &self.stack[slot] {
                    //     Value::Number(n) => self.stack[slot_start + dest as usize] = Value::Number(*n),
                    //     Value::Bool(b) => self.stack[slot_start + dest as usize] = Value::Bool(*b),
                    //     Value::Emptiness => self.stack[slot_start + dest as usize] = Value::Emptiness,
                    //     _ => self.stack[slot_start + dest as usize] = self.stack[slot].clone(),
                    // }
                    // let val = self.stack[slot].clone();
                    // set_register!(dest, val);
                }
                OpCode::Emptiness => {
                    let dest_reg = frame.read_byte();
                    set_register!(slot_start + dest_reg as usize, Value::Emptiness);
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

                    if get_register!(slot_start + condition_reg as usize).is_falsey() {
                        let frame = self.frames.last_mut().unwrap();
                        frame.ip += offset as usize;
                    }
                }
                OpCode::Loop => {
                    let offset = frame.read_u16();
                    frame.ip -= offset as usize;
                }
                OpCode::Halt => break,
                OpCode::Release => {
                    let ret_reg = frame.read_byte();
                    let ret_val = self.stack[slot_start + ret_reg as usize].clone();

                    let finished = self.frames.pop().unwrap();

                    // Write return value into caller's destination register
                    let dest = finished.return_reg;
                    set_register!(slot_start + dest as usize, ret_val);
                }
                OpCode::Cast => {
                    let dest = frame.read_byte();
                    let spell_reg = frame.read_byte();
                    let reg_start = frame.read_byte();

                    let frame_slot_start = frame.slot_start + reg_start as usize;

                    let callee_val = self.stack[frame.slot_start + spell_reg as usize].clone();
                    let spell = match callee_val {
                        Value::Spell(s) => s,
                        Value::Closure(c) => c.spell.clone(),
                        _ => {
                            self.runtime_error("Attempted to cast a non-spell value");
                            return InterpretResult::RuntimeError;
                        }
                    };

                    let closure = ClosureObject {
                        spell: spell,
                        upvalues: vec![],
                    };
                    let new_frame = CallFrame {
                        ip: 0,
                        closure: Rc::new(closure),
                        slot_start: frame_slot_start,
                        return_reg: dest,
                    };
                    self.frames.push(new_frame);
                }
            }
        }

        return InterpretResult::InterpretOk;
    }
}
