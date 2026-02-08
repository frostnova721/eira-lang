# Instruction & OpCode Automation

## Summary

Replaced manual synchronization of `instruction.rs` and `operation.rs` with a single declarative macro that automatically generates all instruction-related code.

## Files Changed

### New Files
- ✅ `src/runtime/instruction_macro.rs` - Single source of truth for all instructions
- ✅ `docs/adding-instructions.md` - Guide for adding new instructions

### Modified Files
- `src/runtime/mod.rs` - Uses new macro, exports `Instruction` and `OpCode`
- `src/runtime/vm.rs` - Updated import path
- `src/frontend/code_gen.rs` - Updated import path, fixed `Halt {}` syntax
- `src/assembler.rs` - Updated import path
- `src/debug.rs` - Updated import path

### Deprecated Files (can be deleted)
- ⚠️ `src/runtime/instruction.rs` - 344 lines → REPLACED
- ⚠️ `src/runtime/operation.rs` - 129 lines → REPLACED

## Benefits

### Before (Manual)
```rust
// In operation.rs - Define OpCode
#[repr(u8)]
pub enum OpCode {
    Add,           // Must remember to add here
    // ... 
}

impl OpCode {
    pub fn to_debug_string(&self) -> String {
        match self {
            OpCode::Add => "OP_ADD",  // And here
            // ...
        }
    }
    
    pub fn inst_len(&self) -> usize {
        match self {
            OpCode::Add => 4,  // And here
            // ...
        }
    }
}

// In instruction.rs - Define Instruction
pub enum Instruction {
    Add { dest: u8, r1: u8, r2: u8 },  // And here
    // ...
}

impl Instruction {
    pub fn len(self) -> usize {
        match self {
            Instruction::Add { .. } => 4,  // And here
            // ...
        }
    }
    
    pub fn to_string(self) -> String {
        match self {
            Instruction::Add { dest, r1, r2 } => format!("ADD {} {} {}", dest, r1, r2),  // And here
            // ...
        }
    }
    
    pub fn get_byte_code(&self) -> Vec<u8> {
        match self {
            Instruction::Add { dest, r1, r2 } => vec![OpCode::Add as u8, *dest, *r1, *r2],  // And here
            // ...
        }
    }
}
```

**Total: 7+ locations to update for ONE instruction** 😫

### After (Automated)
```rust
define_instructions! {
    Add(0, 4) { dest: u8, r1: u8, r2: u8 },  // ONE line!
}
```

**Total: 1 location** 🎉

## What Gets Auto-Generated

From that single line, the macro generates:

1. **OpCode enum variant**
   ```rust
   pub enum OpCode {
       Add = 0,
   }
   ```

2. **Instruction enum variant**
   ```rust
   pub enum Instruction {
       Add { dest: u8, r1: u8, r2: u8 },
   }
   ```

3. **OpCode methods**
   - `from_u8(byte) -> Option<OpCode>`
   - `to_debug_string() -> String`
   - `inst_len() -> usize`

4. **Instruction methods**
   - `len() -> usize`
   - `to_string() -> String`
   - `get_byte_code() -> Vec<u8>`
   - `opcode() -> OpCode`

5. **Trait implementations**
   - `IntoPrimitive` for OpCode
   - `TryFromPrimitive` for OpCode
   - `Debug`, `Clone`, `Copy`, `PartialEq` for both

## Example: Adding a New Instruction

### Before
```bash
# Edit operation.rs - add enum variant
# Edit operation.rs - add to_debug_string match arm
# Edit operation.rs - add inst_len match arm
# Edit instruction.rs - add enum variant
# Edit instruction.rs - add len match arm
# Edit instruction.rs - add to_string match arm
# Edit instruction.rs - add get_byte_code match arm
# Hope you didn't miss anything!
```

### After
```rust
// Just add ONE line:
GetField(26, 5) { dest: u8, sign_reg: u8, field_index: u16 },
```

## Code Reduction

- **Before**: ~473 lines across 2 files
- **After**: ~230 lines in 1 file (including extensive docs)
- **Savings**: ~51% reduction + eliminated manual synchronization

## Safety Improvements

1. **Compile-time size validation**: Size is declared once, verified everywhere
2. **No match arm mismatches**: All generated from same source
3. **Type safety**: Field types checked at compile time
4. **Impossible to desync**: Only one source of truth

## Performance

- ✅ **Zero runtime overhead** - all code generated at compile time
- ✅ Same bytecode format as before
- ✅ Same execution speed
- ✅ Tests pass: bytecode generation verified

## Testing

```rust
#[test]
fn test_instruction_sizes() {
    let instr = Instruction::Add { dest: 0, r1: 1, r2: 2 };
    assert_eq!(instr.len(), 4);
}

#[test]
fn test_bytecode_generation() {
    let instr = Instruction::Add { dest: 1, r1: 2, r2: 3 };
    assert_eq!(instr.get_byte_code(), vec![0, 1, 2, 3]);
}
```

All existing tests pass ✅

## Next Steps

To complete the migration:

1. **Delete old files** (once confident):
   ```bash
   git rm src/runtime/instruction.rs src/runtime/operation.rs
   ```

2. **Add more instructions easily**:
   - For Sign structs: `SignNew`, `GetField`, `SetField`
   - For arrays: `ArrayNew`, `ArrayGet`, `ArraySet`
   - For methods: `CallMethod`

3. **Extend macro if needed**:
   - Add support for u32 fields (for large constants)
   - Add variable-length instructions
   - Auto-generate VM dispatch table

## Migration Checklist

- [x] Create `instruction_macro.rs` with macro
- [x] Define all existing instructions in macro
- [x] Update all imports across codebase
- [x] Fix `Halt {}` struct syntax
- [x] Verify compilation
- [x] Run all tests
- [x] Test VM execution
- [x] Document usage
- [ ] Delete old files (optional, after confidence period)

## Conclusion

The macro-based approach:
- ✅ **Eliminates manual duplication**
- ✅ **Reduces errors**
- ✅ **Makes adding instructions trivial**
- ✅ **Maintains type safety**
- ✅ **Zero performance impact**
- ✅ **Well-documented and testable**

Perfect for a language under active development! 🚀
