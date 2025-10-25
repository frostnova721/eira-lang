use std::{collections::HashMap, rc::Rc};

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
    // slot_start: usize,
    return_reg: u8,
    reg_base: usize,
    caller_reg_base: usize,
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
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(256), // initally
        };

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
            // slot_start: 0,
            return_reg: 0,
            reg_base: 0,
            caller_reg_base: 0,
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

    pub fn start(&mut self) -> InterpretResult {
        macro_rules! set_register {
            ($base:expr, $index:expr, $value:expr) => {{
                let idx = $base + $index as usize;
                if idx >= self.stack.len() {
                    self.stack.resize(idx + 1, Value::Emptiness);
                }
                self.stack[idx] = $value
            }};
        }

        macro_rules! get_register {
            ($base:expr, $index:expr) => {{ self.stack[$base + $index as usize].clone() }};
        }

        macro_rules! binary_op {
            ($frame:expr, $op:tt) => {{
                let (dest, r1, r2) = $frame.read_three_bytes();
                let v1 = get_register!($frame.reg_base, r1);
                let v2 = get_register!($frame.reg_base, r2);
                match (v1, v2) {
                    (Value::Number(n1), Value::Number(n2)) => {
                        set_register!($frame.reg_base, dest, Value::from(n1 $op n2));
                    }
                    _ => {
                        self.runtime_error("Operands should be 2 numbers!");
                        return InterpretResult::RuntimeError;
                    }
                }
            }};
        }

        loop {
            let frame = self.frames.last_mut().unwrap();
            let base = frame.reg_base;
            let op = OpCode::try_from(frame.read_byte()).unwrap();
            match op {
                OpCode::Add => {
                    let (dest, r1, r2) = frame.read_three_bytes();
                    let v1 = get_register!(base, r1);
                    let v2 = get_register!(base, r2);
                    set_register!(
                        base,
                        dest,
                        Value::Number(v1.extract_number().unwrap() + v2.extract_number().unwrap())
                    );
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
                    let v1 = get_register!(base, r1);
                    let v2 = get_register!(base, r2);
                    set_register!(
                        base,
                        dest,
                        Value::String(Rc::new(
                            v1.extract_string().unwrap() + &v2.extract_string().unwrap()
                        ))
                    );
                }
                OpCode::Equal => {
                    let (dest, r1, r2) = frame.read_three_bytes();
                    let a = get_register!(base, r1);
                    let b = get_register!(base, r2);
                    set_register!(base, dest, Value::Bool(a.equals(&b)));
                }
                OpCode::Greater => {
                    binary_op!(frame, >)
                }
                OpCode::Less => {
                    binary_op!(frame, <)
                }
                OpCode::False => {
                    let dest = frame.read_byte();
                    set_register!(base, dest, Value::Bool(false));
                }
                OpCode::True => {
                    let dest = frame.read_byte();
                    set_register!(base, dest, Value::Bool(true));
                }
                OpCode::Negate => {
                    let dest = frame.read_byte();
                    let src_ind = frame.read_byte();
                    let source = get_register!(base, src_ind);
                    match source {
                        Value::Number(n) => set_register!(base, dest, Value::Number(-n)),
                        _ => {
                            self.runtime_error("What???!! Negation needs a number operand.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Not => {
                    let dest = frame.read_byte();
                    let src_ind = frame.read_byte();
                    let source = get_register!(base, src_ind);
                    match source {
                        Value::Bool(b) => set_register!(base, dest, Value::Bool(!b)),
                        _ => {
                            self.runtime_error("What???!! Not needs a boolean operand.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Constant => {
                    let dest = frame.read_byte();
                    let val = frame.read_constant().clone();
                    set_register!(base, dest, val);
                }
                OpCode::Print => {
                    let i = frame.read_byte();
                    let val = get_register!(base, i);
                    print_value(val.clone())
                }
                OpCode::SetGlobal => {
                    let src_reg_ind = frame.read_byte();
                    let var_name_value = frame.read_constant().clone();
                    let value = get_register!(base, src_reg_ind);
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
                            set_register!(base, dest_reg, value.clone());
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
                OpCode::Move => {
                    let dest_slot = frame.read_byte();
                    let src_reg = frame.read_u16();
                    // Move FROM src_reg TO dest_slot (both are register indices in unified model)
                    let val = get_register!(base, src_reg as u8);
                    set_register!(base, dest_slot, val.clone());
                }
                OpCode::Emptiness => {
                    let dest_reg = frame.read_byte();
                    set_register!(base, dest_reg, Value::Emptiness);
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

                    if get_register!(base, condition_reg).is_falsey() {
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
                    let ret_idx = frame.reg_base + ret_reg as usize;
                    let ret_val = self.stack[ret_idx].clone();

                    let finished = self.frames.pop().unwrap();
                    self.stack.truncate(finished.reg_base);

                    let dest_idx = finished.caller_reg_base + finished.return_reg as usize;
                    if dest_idx >= self.stack.len() {
                        self.stack.resize(dest_idx + 1, Value::Emptiness);
                    }
                    self.stack[dest_idx] = ret_val;
                }
                OpCode::Cast => {
                    let dest = frame.read_byte();
                    let spell_reg = frame.read_byte();
                    let reg_start = frame.read_byte();

                    let frame_slot_start = self.stack.len();

                    let callee_idx = frame.reg_base + spell_reg as usize;
                    if callee_idx >= self.stack.len() {
                        self.runtime_error(&format!("Cast: spell register {} (stack index {}) out of bounds (stack size: {})", spell_reg, callee_idx, self.stack.len()));
                        return InterpretResult::RuntimeError;
                    }
                    let callee_val = self.stack[callee_idx].clone();
                    let spell = match callee_val {
                        Value::Closure(c) => c,
                        _ => {
                            self.runtime_error(&format!("Attempted to cast a non-spell value: {:?} at register {} (stack[{}])", callee_val, spell_reg, callee_idx));
                            return InterpretResult::RuntimeError;
                        }
                    };

                    let arity = spell.spell.arity as usize;

                    let upvalues_count = spell.spell.upvalue_count as usize;
                    let total = upvalues_count + arity;

                    // Grow the stack if needed
                    if frame_slot_start + total > self.stack.len() {
                        self.stack
                            .resize(frame_slot_start + total, Value::Emptiness);
                    }

                    for i in 0..upvalues_count {
                        let upval = &spell.upvalues[i];
                        // Get the value from the captured slot
                        let val = if upval.index < self.stack.len() {
                            self.stack[upval.index].clone()
                        } else {
                            upval.closed.clone()
                        };
                        self.stack[frame_slot_start + i] = val;
                    }

                    for i in 0..arity {
                        self.stack[frame_slot_start + upvalues_count + i] =
                            self.stack[frame.reg_base + (reg_start as usize) + i].clone();
                    }

                    let new_frame = CallFrame {
                        ip: 0,
                        closure: spell,
                        // slot_start: frame_slot_start,
                        return_reg: dest,
                        reg_base: frame_slot_start, // Unified: registers start at same place as slots (params are reg 0..arity)
                        caller_reg_base: frame.reg_base,
                    };
                    self.frames.push(new_frame);
                }
            }
        }

        InterpretResult::InterpretOk
    }
}
