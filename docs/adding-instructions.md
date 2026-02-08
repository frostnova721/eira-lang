# Adding New Instructions to Eira

## Quick Guide

All instructions are defined in a single location using the `define_instructions!` macro in `src/runtime/instruction_macro.rs`.

### Step 1: Add Your Instruction

Open `src/runtime/instruction_macro.rs` and add your instruction to the macro invocation:

```rust
define_instructions! {
    // ... existing instructions ...
    
    // Your new instruction
    GetField(26, 5) { dest: u8, sign_reg: u8, field_index: u16 },
    SetField(27, 5) { sign_reg: u8, field_index: u16, value_reg: u8 },
}
```

**Parameters:**
- First number (e.g., `26`): OpCode value (0-255, must be unique)
- Second number (e.g., `5`): Total byte size
  - Formula: 1 (opcode) + sum of field sizes
  - `u8` = 1 byte, `u16` = 2 bytes
  - Example: `5` = 1 (opcode) + 1 (u8) + 1 (u8) + 2 (u16)

### Step 2: Use in Code Generator

In `src/frontend/code_gen.rs`, emit your new instruction:

```rust
// Example: Getting a field from a sign
pub fn gen_get_field(&mut self, sign_reg: u8, field_index: u16) -> u8 {
    let dest_reg = self.allocate_register();
    self.instructions.push(Instruction::GetField {
        dest: dest_reg,
        sign_reg,
        field_index,
    });
    dest_reg
}
```

### Step 3: Handle in VM

In `src/runtime/vm.rs`, add the execution logic:

```rust
// In the main execution loop
match opcode {
    // ... existing opcodes ...
    
    OpCode::GetField => {
        let (dest, sign_reg, field_idx_hi, field_idx_lo) = (
            frame!().read_byte(),
            frame!().read_byte(),
            frame!().read_byte(),
            frame!().read_byte(),
        );
        let field_index = u16::from_le_bytes([field_idx_lo, field_idx_hi]);
        
        let sign = get_register!(frame!().reg_base, sign_reg);
        if let Value::Sign(sign_obj) = sign {
            let field_value = sign_obj.marks[field_index as usize].clone();
            set_register!(frame!().reg_base, dest, field_value);
        } else {
            self.runtime_error("Expected Sign object");
            return InterpretResult::RuntimeError;
        }
    }
}
```

### That's It! 🎉

The macro automatically generates:
- ✅ `OpCode::GetField` enum variant
- ✅ `Instruction::GetField { dest, sign_reg, field_index }` enum variant
- ✅ Bytecode encoding/decoding
- ✅ Debug strings
- ✅ Size calculations
- ✅ All match arms and conversions

## Example: Full Workflow

Let's add a `SignNew` instruction to create sign instances:

**1. Define instruction:**
```rust
// In instruction_macro.rs
SignNew(28, 4) { dest: u8, schema_index: u16 },
```

**2. Emit in code generator:**
```rust
// In code_gen.rs
pub fn gen_sign_new(&mut self, schema_index: u16) -> u8 {
    let dest = self.allocate_register();
    self.instructions.push(Instruction::SignNew {
        dest,
        schema_index,
    });
    dest
}
```

**3. Execute in VM:**
```rust
// In vm.rs
OpCode::SignNew => {
    let dest = frame!().read_byte();
    let schema_index = frame!().read_u16();
    
    let schema = &self.sign_schemas[schema_index as usize];
    let sign = SignObject::new(schema.clone());
    set_register!(frame!().reg_base, dest, Value::Sign(Rc::new(sign)));
}
```

**4. Test:**
```rust
#[test]
fn test_sign_new() {
    let instr = Instruction::SignNew { dest: 5, schema_index: 100 };
    assert_eq!(instr.len(), 4);
    assert_eq!(instr.get_byte_code(), vec![28, 5, 100, 0]); // LE bytes
}
```

## Tips

- **OpCode numbers**: Use sequential numbers, keep them organized
- **Size calculation**: Double-check your math! Test with assertions
- **Field order**: Be consistent (usually: dest first, then sources)
- **Naming**: Use PascalCase for instruction names (e.g., `GetField`, not `get_field`)
- **Testing**: Add unit tests for bytecode generation

## Before the Macro (Don't do this anymore!)

Previously you had to:
1. ❌ Add variant to `OpCode` enum in `operation.rs`
2. ❌ Add variant to `Instruction` enum in `instruction.rs`
3. ❌ Update `OpCode::to_debug_string()` match
4. ❌ Update `OpCode::inst_len()` match
5. ❌ Update `Instruction::len()` match
6. ❌ Update `Instruction::to_string()` match
7. ❌ Update `Instruction::get_byte_code()` match

Now: **Just add one line!** 🚀
