/// # Instruction Definition Macro
///
/// This macro automatically generates both `OpCode` and `Instruction` enums along with
/// all their associated methods from a single declaration. This eliminates the need to
/// manually sync multiple files when adding/modifying instructions.
///
/// ## Benefits
/// - **Single Source of Truth**: Define instructions once
/// - **Auto-generated**: OpCode enum, Instruction enum, bytecode encoding, debug strings
/// - **Type-safe**: Compile-time verification of field types
/// - **Easy to maintain**: Add new instructions by just adding one line
///
/// ## Syntax
/// ```ignore
/// define_instructions! {
///     InstructionName(opcode_value, total_size_in_bytes) {
///         field1: u8,
///         field2: u16,
///         ...
///     },
/// }
/// ```
///
/// ## Parameters
/// - `opcode_value`: The u8 value for this opcode (0-255)
/// - `total_size_in_bytes`: Total size including opcode byte + all field bytes
///   - Example: `Add(0, 4)` = 1 opcode + 3 u8 fields = 4 bytes total
///   - Example: `Constant(10, 4)` = 1 opcode + 1 u8 + 1 u16 = 4 bytes total
///
/// ## Supported Field Types
/// - `u8`: Single byte (register index, small values)
/// - `u16`: Two bytes in little-endian (offsets, constant pool indices)
///
/// ## Generated Code
/// The macro generates:
/// 1. `OpCode` enum with `IntoPrimitive` and `TryFromPrimitive` derives
/// 2. `Instruction` enum with all field combinations
/// 3. `len()` - returns instruction size
/// 4. `to_string()` - human-readable instruction format
/// 5. `get_byte_code()` - converts instruction to bytecode Vec<u8>
/// 6. `opcode()` - extracts the OpCode from an Instruction
/// 7. `inst_len()` - OpCode method returning instruction size
/// 8. `to_debug_string()` - OpCode debug name
///
/// ## Example: Adding a New Instruction
/// ```ignore
/// // Before: Manual work in instruction.rs + operation.rs + multiple match arms
///
/// // After: Just add one line!
/// define_instructions! {
///     // ... existing instructions ...
///     
///     // New instruction for getting struct field
///     GetField(26, 5) { dest: u8, sign_reg: u8, field_index: u16 },
/// }
/// ```
///
/// That's it! Now you can use:
/// ```ignore
/// let instr = Instruction::GetField { 
///     dest: 1, 
///     sign_reg: 2, 
///     field_index: 5 
/// };
/// let bytecode = instr.get_byte_code(); // [26, 1, 2, 5, 0]
/// ```
macro_rules! define_instructions {
    (
        $(
            $instr_name:ident($opcode_val:expr, $size:expr) {
                $( $field:ident : $ty:tt ),* $(,)?
            }
        ),* $(,)?
    ) => {
        // Generate OpCode enum
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
        pub enum OpCode {
            $(
                $instr_name = $opcode_val,
            )*
        }

        impl OpCode {
            pub fn from_u8(byte: u8) -> Option<Self> {
                match byte {
                    $(
                        $opcode_val => Some(OpCode::$instr_name),
                    )*
                    _ => None,
                }
            }

            pub fn to_debug_string(&self) -> String {
                match self {
                    $(
                        OpCode::$instr_name => concat!("OP_", stringify!($instr_name)).to_uppercase(),
                    )*
                }
            }

            pub fn inst_len(&self) -> usize {
                match self {
                    $(
                        OpCode::$instr_name => $size,
                    )*
                }
            }
        }

        // Generate Instruction enum
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Instruction {
            $(
                $instr_name {
                    $( $field : define_instructions!(@type $ty), )*
                },
            )*
        }

        impl Instruction {
            pub fn len(&self) -> usize {
                match self {
                    $(
                        Instruction::$instr_name { .. } => $size,
                    )*
                }
            }

            pub fn to_string(&self) -> String {
                match self {
                    $(
                        Instruction::$instr_name { $($field),* } => {
                            let name = stringify!($instr_name).to_uppercase();
                            define_instructions!(@format_instr name, $($field),*)
                        }
                    )*
                }
            }

            pub fn get_byte_code(&self) -> Vec<u8> {
                match self {
                    $(
                        Instruction::$instr_name { $($field),* } => {
                            let bytes = vec![OpCode::$instr_name as u8];
                            $(
                                define_instructions!(@encode_field bytes, $field, $ty);
                            )*
                            bytes
                        }
                    )*
                }
            }

            pub fn opcode(&self) -> OpCode {
                match self {
                    $(
                        Instruction::$instr_name { .. } => OpCode::$instr_name,
                    )*
                }
            }
        }
    };

    // Type helper
    (@type u8) => { u8 };
    (@type u16) => { u16 };

    // Helper: Format instruction string
    (@format_instr $name:expr,) => {
        $name.to_string()
    };
    (@format_instr $name:expr, $field:expr) => {
        format!("{} {}", $name, $field)
    };
    (@format_instr $name:expr, $field1:expr, $field2:expr) => {
        format!("{} {} {}", $name, $field1, $field2)
    };
    (@format_instr $name:expr, $field1:expr, $field2:expr, $field3:expr) => {
        format!("{} {} {} {}", $name, $field1, $field2, $field3)
    };

    // Helper: Encode field to bytes
    (@encode_field $bytes:ident, $field:expr, u8) => {
        let $bytes = { let mut v = $bytes; v.push(*$field); v };
    };
    (@encode_field $bytes:ident, $field:expr, u16) => {
        let $bytes = { let mut v = $bytes; v.extend_from_slice(&$field.to_le_bytes()); v };
    };
}

// Usage example - define all your instructions here
define_instructions! {
    // Arithmetic (3-register format: dest, r1, r2)
    Add(0, 4) { dest: u8, r1: u8, r2: u8 },
    Subtract(1, 4) { dest: u8, r1: u8, r2: u8 },
    Multiply(2, 4) { dest: u8, r1: u8, r2: u8 },
    Divide(3, 4) { dest: u8, r1: u8, r2: u8 },
    Mod(4, 4) { dest: u8, r1: u8, r2: u8 },

    // Comparison
    Equal(5, 4) { dest: u8, r1: u8, r2: u8 },
    Greater(6, 4) { dest: u8, r1: u8, r2: u8 },
    Less(7, 4) { dest: u8, r1: u8, r2: u8 },

    // Unary operations
    Negate(8, 3) { dest: u8, r1: u8 },
    Not(9, 3) { dest: u8, r1: u8 },

    // Constants and values
    Constant(10, 4) { dest: u8, const_index: u16 },
    True(11, 2) { dest: u8 },
    False(12, 2) { dest: u8 },
    Emptiness(13, 2) { dest: u8 },

    // String operations
    Concat(14, 4) { dest: u8, r1: u8, r2: u8 },

    // I/O
    Print(15, 2) { r1: u8 },

    // Globals
    SetGlobal(16, 4) { src_reg: u8, const_index: u16 },
    GetGlobal(17, 4) { dest: u8, const_index: u16 },

    // Locals/Registers
    Move(18, 4) { dest: u8, source: u16 },
    PopStack(19, 3) { pop_count: u16 },

    // Control flow
    Jump(20, 3) { offset: u16 },
    JumpIfFalse(21, 4) { condition_reg: u8, offset: u16 },
    Loop(22, 3) { offset: u16 },

    // Function calls
    Cast(23, 4) { dest: u8, spell_reg: u8, reg_start: u8 },
    Release(24, 2) { dest: u8 },

    // Termination
    Halt(25, 1) {},

    // Sign Stuff
    NewSign(26,1) { dest: u8, const_idx: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_sizes() {
        let instr = Instruction::Add { dest: 0, r1: 1, r2: 2 };
        assert_eq!(instr.len(), 4); // opcode + 3 u8s

        let instr = Instruction::Constant { dest: 0, const_index: 100 };
        assert_eq!(instr.len(), 4); // opcode + u8 + u16

        let instr = Instruction::Halt {};
        assert_eq!(instr.len(), 1); // just opcode
    }

    #[test]
    fn test_bytecode_generation() {
        let instr = Instruction::Add { dest: 1, r1: 2, r2: 3 };
        let bytes = instr.get_byte_code();
        assert_eq!(bytes, vec![0, 1, 2, 3]);

        let instr = Instruction::Constant { dest: 5, const_index: 256 };
        let bytes = instr.get_byte_code();
        assert_eq!(bytes, vec![10, 5, 0, 1]); // 256 in LE = [0, 1]
    }

    #[test]
    fn test_opcode_conversions() {
        assert_eq!(OpCode::from_u8(0), Some(OpCode::Add));
        assert_eq!(OpCode::from_u8(25), Some(OpCode::Halt));
        assert_eq!(OpCode::from_u8(255), None);
    }
}
