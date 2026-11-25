use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    SpellObject,
    compiler::compiler::CompiledCode,
    runtime::{
        OpCode,
        error::{RuntimeError, RuntimeLocation, TraceFrame},
    },
    values::{
        Value,
        deck::DeckObject,
        iterator::{IteratorObject, IteratorState},
        native_spell::dispatch,
        print_value,
        sign::SignObject,
        spell::{ClosureObject, UpValue},
    },
};

/// Nested casts allowed before the VM reports a stack overflow.
pub const MAX_FRAMES: usize = 10_000;

pub enum InterpretResult {
    CompileError,
    RuntimeError(RuntimeError),
    InterpretOk,
    Returned(Value),
}

#[derive(Debug)]
pub enum ReturnTarget {
    Module,
    Register { base: usize, reg: u8 },
    Host,
}

#[derive(Debug)]
struct CallFrame {
    ip: usize,
    closure: Rc<ClosureObject>,
    // slot_start: usize,
    return_target: ReturnTarget,
    reg_base: usize,
    // caller_reg_base: usize,

    // Track which registers in this frame are upvalues and where they point in parent
    upvalue_mappings: Vec<(usize, usize)>, // (local_reg, parent_stack_idx)
}

// Operand reads trust the bytecode layout produced by the compiler. The dispatch
// loop checks that each instruction starts inside the bytecode.
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

    /// Source location of the instruction this frame is executing (or, for a caller
    /// frame, the cast it is waiting on). `ip` is always past the opcode byte.
    fn location(&self) -> Option<RuntimeLocation> {
        self.closure
            .spell
            .lines
            .lookup(self.ip.saturating_sub(1))
            .map(|entry| RuntimeLocation {
                file: entry.file.to_string(),
                line: entry.line,
                column: entry.column,
            })
    }
}

/// Converts a deck index value to a position, rejecting negative or fractional numbers.
fn to_index(value: &Value) -> Result<usize, String> {
    match value {
        Value::Number(n) if n.is_finite() && *n >= 0.0 && n.fract() == 0.0 => Ok(*n as usize),
        Value::Number(n) => Err(format!(
            "Deck index must be a whole, non-negative number. Got {}.",
            n
        )),
        other => Err(format!("Deck index must be a number. Got {:?}.", other)),
    }
}

pub struct EiraVM {
    frames: Vec<CallFrame>,
    initialized: bool,

    globals: HashMap<String, Value>,
    pub stack: Vec<Value>,
}

impl EiraVM {
    pub fn init(compiled_code: CompiledCode) -> Self {
        let mut vm = EiraVM {
            globals: HashMap::new(),
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(256), // initally
            initialized: false,
        };

        let closure = ClosureObject {
            spell: Rc::new(SpellObject {
                arity: 0,
                bytecode: compiled_code.bytecode,
                constants: compiled_code.constants,
                name: None,
                upvalue_count: 0,
                lines: compiled_code.lines,
            }),
            upvalues: vec![],
        };

        let frame = CallFrame {
            closure: Rc::new(closure),
            ip: 0,
            // slot_start: 0,
            return_target: ReturnTarget::Module,
            reg_base: 0,
            // caller_reg_base: 0,
            upvalue_mappings: vec![],
        };

        vm.frames.push(frame);

        vm
    }

    /// Builds an error carrying the location of every active frame, innermost first.
    fn error_here(&self, msg: impl Into<String>) -> RuntimeError {
        let trace = self
            .frames
            .iter()
            .rev()
            .map(|frame| TraceFrame {
                spell: frame.closure.spell.name.clone(),
                location: frame.location(),
            })
            .collect();
        RuntimeError {
            message: msg.into(),
            trace,
        }
    }

    /// Invoke a function from external code with spell name and its arguments.
    pub fn cast(&mut self, spell: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        if !self.initialized {
            return Err(RuntimeError::new(
                "Run module initialization before invoking a spell.",
            ));
        }

        if !self.frames.is_empty() {
            return Err(RuntimeError::new(
                "The VM is already executing or has unfinished frames.",
            ));
        }

        let closure = match self.globals.get(spell) {
            Some(Value::Closure(closure)) => Rc::clone(closure),
            _ => {
                return Err(RuntimeError::new(format!(
                    "Spell '{spell}' not found or is not a closure."
                )));
            }
        };

        let arity = closure.spell.arity as usize;
        if args.len() != arity {
            return Err(RuntimeError::new(format!(
                "Expected {arity} arguments, but got {}.",
                args.len()
            )));
        }

        let stack_base = self.stack.len();
        let frame_depth = self.frames.len();

        // Match the register layout used by OpCode::Cast.
        for capture in &closure.upvalues {
            self.stack.push(capture.closed.borrow().clone());
        }
        self.stack.extend(args);

        self.frames.push(CallFrame {
            ip: 0,
            closure,
            reg_base: stack_base,
            return_target: ReturnTarget::Host,
            upvalue_mappings: vec![],
        });

        let outcome = self.start();

        // Clean up temporary invocation state, including on runtime errors.
        // This does not undo mutations to globals or shared objects.
        self.frames.truncate(frame_depth);
        self.stack.truncate(stack_base);

        match outcome {
            InterpretResult::Returned(value) => Ok(value),
            InterpretResult::RuntimeError(error) => Err(error),
            InterpretResult::CompileError => Err(RuntimeError::new("Compile error occurred.")),
            InterpretResult::InterpretOk => Err(RuntimeError::new(
                "Invocation ended without returning to the host.",
            )),
        }
    }

    pub fn start(&mut self) -> InterpretResult {
        if self.frames.is_empty() {
            return InterpretResult::InterpretOk;
        }

        // Stops execution with a runtime error at the current instruction.
        macro_rules! fail {
            ($($arg:tt)*) => {{
                let error = self.error_here(format!($($arg)*));
                return InterpretResult::RuntimeError(error);
            }};
        }

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
            ($base:expr, $index:expr) => {{
                let idx = $base + $index as usize;
                if idx >= self.stack.len() {
                    fail!(
                        "Register {} was read before being written. This is a compiler bug, please report it.",
                        $index
                    );
                }
                &self.stack[idx]
            }};
        }

        // The loop only runs while a frame exists (checked at the top of each iteration).
        macro_rules! frame {
            () => {
                self.frames.last_mut().unwrap()
            };
        }

        macro_rules! binary_op {
            ($op:tt) => {{
                let (dest, r1, r2) = frame!().read_three_bytes();
                let base = frame!().reg_base;
                let v1 = get_register!(base, r1);
                let v2 = get_register!(base, r2);
                match (v1, v2) {
                    (Value::Number(n1), Value::Number(n2)) => {
                        let r = n1 $op n2;
                        set_register!(base, dest, Value::from(r));
                    }
                    _ => {
                        let msg = format!("Operands should be 2 numbers! Got {:?} and {:?}", v1, v2);
                        fail!("{}", msg);
                    }
                }
            }};
        }

        // Keep this on to profile execution
        // let mut instruction_count: u32 = 0;

        loop {
            if self.frames.is_empty() {
                return InterpretResult::InterpretOk;
            }
            let base = frame!().reg_base;
            if frame!().ip >= frame!().closure.spell.bytecode.len() {
                fail!("Execution ran past the end of the bytecode. This is a compiler bug, please report it.");
            }
            let byte = frame!().read_byte();
            let op = match OpCode::try_from(byte) {
                Ok(op) => op,
                Err(_) => fail!("Invalid opcode {} encountered. The bytecode is corrupted.", byte),
            };
            // instruction_count += 1;
            match op {
                OpCode::Add => binary_op!(+),
                OpCode::Subtract => binary_op!(-),
                OpCode::Divide => binary_op!(/),
                OpCode::Multiply => binary_op!(*),
                OpCode::Mod => binary_op!(%),
                OpCode::Concat => {
                    let (dest, r1, r2) = frame!().read_three_bytes();
                    let v1 = get_register!(base, r1);
                    let v2 = get_register!(base, r2);
                    let r = match (v1, v2) {
                        (Value::String(a), Value::String(b)) => {
                            let mut s = String::with_capacity(a.len() + b.len());
                            s.push_str(a);
                            s.push_str(b);
                            s
                        }
                        _ => {
                            let msg = format!("Concatenation needs 2 texts! Got {:?} and {:?}", v1, v2);
                            fail!("{}", msg);
                        }
                    };
                    set_register!(base, dest, Value::String(Rc::new(r)));
                }
                OpCode::Equal => {
                    let (dest, r1, r2) = frame!().read_three_bytes();
                    let a = get_register!(base, r1);
                    let b = get_register!(base, r2);
                    let r = a.equals(&b);
                    set_register!(base, dest, Value::Bool(r));
                }
                OpCode::Greater => {
                    binary_op!(>)
                }
                OpCode::Less => {
                    binary_op!(<)
                }
                OpCode::False => {
                    let dest = frame!().read_byte();
                    set_register!(base, dest, Value::Bool(false));
                }
                OpCode::True => {
                    let dest = frame!().read_byte();
                    set_register!(base, dest, Value::Bool(true));
                }
                OpCode::Negate => {
                    let dest = frame!().read_byte();
                    let src_ind = frame!().read_byte();
                    match get_register!(base, src_ind) {
                        Value::Number(n) => {
                            let num = *n;
                            set_register!(base, dest, Value::Number(-num));
                        }
                        _ => fail!("What???!! Negation needs a number operand."),
                    }
                }
                OpCode::Not => {
                    let dest = frame!().read_byte();
                    let src_ind = frame!().read_byte();
                    match get_register!(base, src_ind) {
                        Value::Bool(b) => {
                            let boo = *b;
                            set_register!(base, dest, Value::Bool(!boo));
                        }
                        _ => fail!("What???!! Not needs a boolean operand."),
                    }
                }
                OpCode::Constant => {
                    let dest = frame!().read_byte();
                    let cval = frame!().read_constant().clone();
                    match cval {
                        Value::Closure(c) => {
                            // Eagerly bind upvalues: capture from current frame into a fresh ClosureObject
                            let mut new_upvalues: Vec<UpValue> = Vec::with_capacity(c.upvalues.len());
                            for u in c.upvalues.iter() {
                                let captured = self
                                    .stack
                                    .get(base + u.index)
                                    .cloned()
                                    .unwrap_or(Value::Emptiness);
                                new_upvalues.push(UpValue {
                                    index: u.index,
                                    depth: u.depth,
                                    closed: std::cell::RefCell::new(captured),
                                });
                            }

                            let new_closure = ClosureObject {
                                spell: c.spell.clone(),
                                upvalues: new_upvalues,
                            };
                            set_register!(base, dest, Value::Closure(Rc::new(new_closure)));
                        }
                        other => {
                            set_register!(base, dest, other);
                        }
                    }
                }
                OpCode::Print => {
                    let i = frame!().read_byte();
                    let val = get_register!(base, i);
                    print_value(val.clone())
                }
                OpCode::SetGlobal => {
                    let src_reg_ind = frame!().read_byte();
                    let var_name_value = frame!().read_constant().clone();
                    let value = get_register!(base, src_reg_ind).clone();
                    if let Value::String(name) = var_name_value {
                        self.globals.insert(name.to_string(), value);
                    } else {
                        fail!("Fatal: A string was expected for the global variable name.");
                    }
                }
                OpCode::GetGlobal => {
                    let dest_reg = frame!().read_byte();
                    let val = frame!().read_constant().clone();
                    if let Value::String(name) = val {
                        match self.globals.get(name.as_str()) {
                            Some(value) => {
                                let value = value.clone();
                                set_register!(base, dest_reg, value);
                            }
                            None => fail!("The mark '{}' was undefined", name),
                        }
                    } else {
                        fail!("Fatal: A string was expected for the global variable name.");
                    }
                }
                OpCode::Move => {
                    let dest_slot = frame!().read_byte();
                    let src_reg = frame!().read_u16();
                    // Move FROM src_reg TO dest_slot (both are register indices)
                    let val = get_register!(base, src_reg).clone();
                    set_register!(base, dest_slot, val);
                }
                OpCode::Emptiness => {
                    let dest_reg = frame!().read_byte();
                    set_register!(base, dest_reg, Value::Emptiness);
                }
                OpCode::PopStack => {
                    let count = frame!().read_u16() as usize;
                    let new_len = self.stack.len().saturating_sub(count);
                    self.stack.truncate(new_len);
                }
                OpCode::Jump => {
                    let offset = frame!().read_u16();
                    // An out-of-range target is caught at the top of the loop
                    frame!().ip += offset as usize;
                }
                OpCode::JumpIfFalse => {
                    let condition_reg = frame!().read_byte();
                    let offset = frame!().read_u16();

                    if get_register!(base, condition_reg).is_falsey() {
                        frame!().ip += offset as usize;
                    }
                }
                OpCode::JumpIfTrue => {
                    let condition_reg = frame!().read_byte();
                    let offset = frame!().read_u16();

                    if get_register!(base, condition_reg).is_truthy() {
                        frame!().ip += offset as usize;
                    }
                }
                OpCode::Loop => {
                    let offset = frame!().read_u16();
                    match frame!().ip.checked_sub(offset as usize) {
                        Some(ip) => frame!().ip = ip,
                        None => fail!("Loop jumped before the start of the bytecode. This is a compiler bug, please report it."),
                    }
                }
                OpCode::Halt => {
                    self.frames.pop();
                    self.initialized = true;
                    break;
                }
                OpCode::Release => {
                    let ret_reg = frame!().read_byte();
                    let ret_val = get_register!(base, ret_reg).clone();

                    let Some(finished) = self.frames.pop() else {
                        fail!("Release without an active spell.");
                    };

                    // Sync upvalue changes back to parent frame before truncating stack
                    for (local_reg, parent_stack_idx) in &finished.upvalue_mappings {
                        let local_stack_idx = finished.reg_base + local_reg;
                        if local_stack_idx < self.stack.len()
                            && *parent_stack_idx < self.stack.len()
                        {
                            if let Some(upvalue) = finished.closure.upvalues.get(*local_reg) {
                                let val = self.stack[local_stack_idx].clone();
                                *upvalue.closed.borrow_mut() = val;
                            }
                        }
                    }

                    self.stack.truncate(finished.reg_base);

                    match finished.return_target {
                        ReturnTarget::Register { base, reg } => {
                            set_register!(base, reg, ret_val);
                        }
                        ReturnTarget::Host => {
                            return InterpretResult::Returned(ret_val);
                        }
                        ReturnTarget::Module => {
                            return InterpretResult::InterpretOk;
                        }
                    }
                }
                OpCode::Cast => {
                    let dest = frame!().read_byte();
                    let spell_reg = frame!().read_byte();
                    let reg_start = frame!().read_byte();

                    if self.frames.len() >= MAX_FRAMES {
                        fail!(
                            "Stack overflow: more than {} nested casts. Is a spell recursing forever?",
                            MAX_FRAMES
                        );
                    }

                    let spell = match get_register!(base, spell_reg) {
                        Value::Closure(c) => c.clone(),
                        other => {
                            let msg = format!("Attempted to cast a non-spell value: {:?}", other);
                            fail!("{}", msg);
                        }
                    };

                    let arity = spell.spell.arity as usize;
                    let arg_start = base + reg_start as usize;
                    if arity > 0 && arg_start + arity > self.stack.len() {
                        fail!(
                            "Cast reagents (registers {}..{}) were not set. This is a compiler bug, please report it.",
                            reg_start,
                            reg_start as usize + arity
                        );
                    }

                    let frame_slot_start = self.stack.len();
                    let upvalues_count = spell.upvalues.len();
                    let total = upvalues_count + arity;

                    // Grow the stack if needed
                    self.stack
                        .resize(frame_slot_start + total, Value::Emptiness);

                    // Track upvalue mappings for this frame (unused with eager capture)
                    let upvalue_mappings = Vec::new();
                    // Populate upvalue registers from eagerly captured values
                    for (i, upvalue) in spell.upvalues.iter().enumerate() {
                        self.stack[frame_slot_start + i] = upvalue.closed.borrow().clone();
                    }

                    for i in 0..arity {
                        self.stack[frame_slot_start + upvalues_count + i] =
                            self.stack[arg_start + i].clone();
                    }

                    let new_frame = CallFrame {
                        ip: 0,
                        closure: spell,
                        // slot_start: frame_slot_start,
                        return_target: ReturnTarget::Register { base, reg: dest },
                        reg_base: frame_slot_start, // Unified: registers start at same place as slots (params are reg 0..arity)
                        // caller_reg_base: frame!().reg_base,
                        upvalue_mappings,
                    };
                    self.frames.push(new_frame);
                }
                OpCode::NewSign => {
                    let dest = frame!().read_byte();
                    let value = frame!().read_byte();
                    let schema = match get_register!(base, value) {
                        Value::SignSchema(sc) => sc.clone(),
                        other => {
                            let msg = format!("Expected a sign schema to draw from, got {:?}.", other);
                            fail!("{}", msg);
                        }
                    };

                    let sign = SignObject::new(schema);
                    set_register!(base, dest, Value::Sign(Rc::new(RefCell::new(sign))));
                }
                OpCode::SetField => {
                    let sign_reg = frame!().read_byte();
                    let field_name_idx = frame!().read_u16();
                    let val_reg = frame!().read_byte();

                    let val = get_register!(base, val_reg).clone();

                    let result = match get_register!(base, sign_reg) {
                        Value::Sign(s) => s.borrow_mut().set_field(field_name_idx as usize, val),
                        Value::Emptiness => Err("Cannot set a field on an empty value.".to_string()),
                        other => Err(format!("SET_FIELD needs a 'Sign' value, got {:?}.", other)),
                    };
                    if let Err(e) = result {
                        fail!("{}", e);
                    }
                }
                OpCode::GetField => {
                    let dest = frame!().read_byte();
                    let sign_reg = frame!().read_byte();
                    let field_name = frame!().read_u16();

                    let val = match get_register!(base, sign_reg) {
                        Value::Sign(s) => s.borrow().get_field(field_name as usize),
                        Value::Emptiness => fail!("Cannot read a field of an empty value."),
                        _ => fail!("GET_FIELD Operation was used with a non 'Sign' value"),
                    };
                    match val {
                        Some(val) => set_register!(base, dest, val),
                        None => fail!("Field index {} out of bounds.", field_name),
                    }
                }
                OpCode::SafeGetField => {
                    let dest = frame!().read_byte();
                    let sign_reg = frame!().read_byte();
                    let field_name = frame!().read_u16();

                    let val = match get_register!(base, sign_reg) {
                        Value::Emptiness => Some(Value::Emptiness),
                        Value::Sign(s) => s.borrow().get_field(field_name as usize),
                        _ => fail!("SAFE_GET_FIELD Operation was used with a non 'Sign' value"),
                    };
                    match val {
                        Some(val) => set_register!(base, dest, val),
                        None => fail!("Field index {} out of bounds.", field_name),
                    }
                }
                OpCode::NewDeck => {
                    let reg = frame!().read_byte();
                    let start = frame!().read_byte() as usize;
                    let count = frame!().read_byte() as usize;

                    let mut values: Vec<Value> = Vec::with_capacity(count);
                    for i in start..start + count {
                        values.push(get_register!(base, i).clone());
                    }

                    set_register!(
                        base,
                        reg,
                        Value::Deck(Rc::new(DeckObject::new(values, None)))
                    );
                }
                OpCode::NewFixedDeck => {
                    let reg = frame!().read_byte();
                    let start = frame!().read_byte() as usize;
                    let count = frame!().read_byte() as usize;
                    let capacity = frame!().read_u16() as usize;

                    let mut values: Vec<Value> = Vec::with_capacity(capacity.max(count));
                    for i in start..start + count {
                        values.push(get_register!(base, i).clone());
                    }

                    set_register!(
                        base,
                        reg,
                        Value::Deck(Rc::new(DeckObject::new(values, Some(capacity))))
                    );
                }
                OpCode::AddToDeck => {
                    let deck_reg = frame!().read_byte();
                    let position = frame!().read_byte();
                    let value_reg = frame!().read_byte();
                    let val = get_register!(base, value_reg).clone();
                    let idx = match to_index(get_register!(base, position)) {
                        Ok(idx) => idx,
                        Err(e) => fail!("{}", e),
                    };

                    let d = match get_register!(base, deck_reg) {
                        Value::Deck(d) => d.clone(),
                        _ => fail!("Value is not a Deck to perform 'ADD_TO_DECK Operation'"),
                    };

                    let len = d.items.borrow().len();

                    if let Some(cap) = d.capacity {
                        if idx >= cap {
                            fail!("Index out of bounds while adding element to a deck. Tried to add at {} while deck capacity is {}.", idx, cap);
                        }
                    }

                    if idx > len {
                        fail!("Index out of bounds while adding element to a deck. Tried to add at {} while deck size is {}.", idx, len);
                    } else if idx == len {
                        d.items.borrow_mut().push(val);
                    } else {
                        d.items.borrow_mut()[idx] = val;
                    }
                }

                OpCode::ExtractFromDeck => {
                    let dest = frame!().read_byte();
                    let deck_reg = frame!().read_byte();
                    let index = frame!().read_byte();

                    let idx = match to_index(get_register!(base, index)) {
                        Ok(idx) => idx,
                        Err(e) => fail!("{}", e),
                    };
                    let d = match get_register!(base, deck_reg) {
                        Value::Deck(d) => d.clone(),
                        _ => fail!("Value is not a Deck to perform 'EXTRACT_FROM_DECK' Operation'"),
                    };

                    let item = d.items.borrow().get(idx).cloned();
                    match item {
                        Some(val) => set_register!(base, dest, val),
                        None => fail!(
                            "Index out of bounds while extracting element from a deck. Tried to access {} while deck size is {}.",
                            idx,
                            d.items.borrow().len()
                        ),
                    }
                }
                OpCode::NewVault => {
                    let dest = frame!().read_byte();
                    let limit = frame!().read_constant().extract_number();
                    let hint = frame!().read_u16() as usize;
                    let Some(limit) = limit else {
                        fail!("Vault capacity constant is not a number. This is a compiler bug, please report it.");
                    };
                    let limit = if limit < 0.0 { None } else { Some(limit as usize) };
                    match crate::values::vault::VaultObject::new(limit, hint) {
                        Ok(v) => set_register!(base, dest, Value::Vault(Rc::new(v))),
                        Err(e) => fail!("{}", e),
                    }
                }
                OpCode::VaultSet => {
                    let (object, key, value) = frame!().read_three_bytes();
                    let key = get_register!(base, key).clone();
                    let value = get_register!(base, value).clone();
                    let result = match get_register!(base, object) {
                        Value::Vault(v) => v.insert(&key, value),
                        _ => Err("Expected Vault for indexed assignment.".into()),
                    };
                    if let Err(e) = result {
                        fail!("{}", e);
                    }
                }
                OpCode::VaultGet => {
                    let (dest, object, key) = frame!().read_three_bytes();
                    let key = get_register!(base, key).clone();
                    let result = match get_register!(base, object) {
                        Value::Vault(v) => v.get(&key),
                        _ => Err("Expected Vault for indexed lookup.".into()),
                    };
                    match result {
                        Ok(value) => set_register!(base, dest, value),
                        Err(e) => fail!("{}", e),
                    }
                }
                OpCode::IsEmptiness => {
                    let dest = frame!().read_byte();
                    let r1 = frame!().read_byte();

                    let is_empty = get_register!(base, r1).is_emptiness();
                    set_register!(base, dest, Value::Bool(is_empty));
                }
                OpCode::AssertSafe => {
                    let r1 = frame!().read_byte();
                    if get_register!(base, r1).is_emptiness() {
                        fail!("Safe Assertion failed: value is empty.");
                    }
                }

                OpCode::NativeCast => {
                    let dest = frame!().read_byte();
                    let spell = frame!().read_constant().clone();
                    let arg_start = frame!().read_byte();
                    let argc = frame!().read_byte() as usize;

                    let Value::NativeSpell(ns) = spell else {
                        fail!("Expected a NativeSpell value to be casted!");
                    };

                    // Native handlers index their arguments directly, so make sure they exist.
                    let start_idx = base + arg_start as usize;
                    if start_idx + argc > self.stack.len() {
                        fail!(
                            "Native spell reagents (registers {}..{}) were not set. This is a compiler bug, please report it.",
                            arg_start,
                            arg_start as usize + argc
                        );
                    }

                    match dispatch(self, ns, start_idx, argc) {
                        Ok(v) => set_register!(base, dest, v),
                        Err(e) => fail!("Error running a native spell.\n{}", e),
                    }
                }

                OpCode::CreateRange => {
                    let dest = frame!().read_byte();
                    let start_reg = frame!().read_byte();
                    let end_reg = frame!().read_byte();

                    let start_val = get_register!(base, start_reg).clone();
                    let end_val = get_register!(base, end_reg).clone();

                    match (start_val, end_val) {
                        (Value::Number(start), Value::Number(end)) => {
                            set_register!(base, dest, Value::Range(start, end));
                        }
                        _ => fail!("CreateRange: Both start and end values must be numbers."),
                    }
                }

                OpCode::GetRangeStart => {
                    let dest = frame!().read_byte();
                    let range_reg = frame!().read_byte();

                    match get_register!(base, range_reg).clone() {
                        Value::Range(start, _) => set_register!(base, dest, Value::Number(start)),
                        _ => fail!("GetRangeStart: Value is not a Range."),
                    }
                }
                OpCode::GetRangeEnd => {
                    let dest = frame!().read_byte();
                    let range_reg = frame!().read_byte();

                    match get_register!(base, range_reg).clone() {
                        Value::Range(_, end) => set_register!(base, dest, Value::Number(end)),
                        _ => fail!("GetRangeEnd: Value is not a Range."),
                    }
                }
                OpCode::GetIterator => {
                    let dest = frame!().read_byte();
                    let iterable_reg = frame!().read_byte();

                    let iter = match get_register!(base, iterable_reg).clone() {
                        Value::Range(start, end) => IteratorObject {
                            state: IteratorState::Range {
                                current: start,
                                end,
                            },
                        },
                        Value::Deck(d) => IteratorObject {
                            state: IteratorState::Deck { deck: d, index: 0 },
                        },
                        _ => fail!("GetIterator: Iterator for Value is not defined (not a Range or Deck)."),
                    };

                    set_register!(base, dest, Value::Iterator(Rc::new(RefCell::new(iter))));
                }

                OpCode::IterNext => {
                    let dest = frame!().read_byte();
                    let iterator_reg = frame!().read_byte();
                    let jump_offset = frame!().read_u16();

                    let iter = match get_register!(base, iterator_reg) {
                        Value::Iterator(iter) => iter.clone(),
                        _ => fail!("IterNext: Value is not an Iterator."),
                    };

                    let next = {
                        let mut iter_mut = iter.borrow_mut();
                        match &mut iter_mut.state {
                            IteratorState::Range { current, end } => {
                                if *current <= *end {
                                    let value = Value::Number(*current);
                                    *current += 1.0;
                                    Some(value)
                                } else {
                                    None
                                }
                            }
                            IteratorState::Deck { deck, index } => {
                                let item = deck.items.borrow().get(*index).cloned();
                                if item.is_some() {
                                    *index += 1;
                                }
                                item
                            }
                        }
                    };

                    match next {
                        Some(value) => set_register!(base, dest, value),
                        // End of iteration
                        None => frame!().ip += jump_offset as usize,
                    }
                }
            }
        }
        // println!("Program completed after {} instructions.", instruction_count);
        InterpretResult::InterpretOk
    }
}
