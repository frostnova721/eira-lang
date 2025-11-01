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

    // Track which registers in this frame are upvalues and where they point in parent
    upvalue_mappings: Vec<(usize, usize)>, // (local_reg, parent_stack_idx)
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
            upvalue_mappings: vec![],
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
            ($base:expr, $index:expr) => {{ &self.stack[$base + $index as usize] }};
        }

        macro_rules! binary_op {
            ($frame:expr, $op:tt) => {{
                let (dest, r1, r2) = $frame.read_three_bytes();
                let v1 = get_register!($frame.reg_base, r1);
                let v2 = get_register!($frame.reg_base, r2);
                match (v1, v2) {
                    (Value::Number(n1), Value::Number(n2)) => {
                        let r = n1 $op n2;
                        set_register!($frame.reg_base, dest, Value::from(r));
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
                    binary_op!(frame, +)
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
                    let r = v1.extract_string().unwrap() + &v2.extract_string().unwrap();
                    set_register!(base, dest, Value::String(Rc::new(r)));
                }
                OpCode::Equal => {
                    let (dest, r1, r2) = frame.read_three_bytes();
                    let a = get_register!(base, r1);
                    let b = get_register!(base, r2);
                    let r = a.equals(&b);
                    set_register!(base, dest, Value::Bool(r));
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
                        Value::Number(n) => {
                            let num = *n;
                            set_register!(base, dest, Value::Number(-num));
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
                    let source = get_register!(base, src_ind);
                    match source {
                        Value::Bool(b) => {
                            let boo = *b;
                            set_register!(base, dest, Value::Bool(!boo));
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
                    // Move FROM src_reg TO dest_slot (both are register indices)
                    let val = get_register!(base, src_reg as u8).clone();
                    set_register!(base, dest_slot, val);
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

                    // Sync upvalue changes back to parent frame before truncating stack
                    for (local_reg, parent_stack_idx) in &finished.upvalue_mappings {
                        let local_stack_idx = finished.reg_base + local_reg;
                        if local_stack_idx < self.stack.len()
                            && *parent_stack_idx < self.stack.len()
                        {
                            self.stack[*parent_stack_idx] = self.stack[local_stack_idx].clone();
                        }
                    }

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

                    // Track upvalue mappings for this frame
                    let mut upvalue_mappings = Vec::new();
                    for i in 0..upvalues_count {
                        let upval = &spell.upvalues[i];
                        // Get the value from the captured slot (relative to caller's frame base)
                        let upval_stack_idx = frame.reg_base + upval.index;
                        let val = if upval_stack_idx < self.stack.len() {
                            match &self.stack[upval_stack_idx] {
                                Value::Closure(c) => &c.spell.constants[upval.index as usize],
                                _ => {
                                    // this shouldnt really be executed tbh, but just in case
                                    println!(
                                        "Warning: upvalue at stack index {} is not a Closure!",
                                        upval_stack_idx
                                    );
                                    &self.stack[upval_stack_idx]
                                }
                            }
                        } else {
                            &upval.closed
                        };
                        self.stack[frame_slot_start + i] = val.clone();
                        // Track that local register i points to parent's upval_stack_idx
                        upvalue_mappings.push((i, upval_stack_idx));
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
                        upvalue_mappings,
                    };
                    self.frames.push(new_frame);
                }
            }
        }

        InterpretResult::InterpretOk
    }
}
