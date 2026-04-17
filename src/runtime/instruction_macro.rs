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
    (@format_instr $name:expr, $field1:expr, $field2:expr, $field3:expr, $field4:expr) => {
        format!("{} {} {} {} {}", $name, $field1, $field2, $field3, $field4)
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

    // Sign Stuff. schema_reg is the register which has the schema
    NewSign(26,4) { dest: u8, schema_reg: u8 },

    // Set a field to a sign. [field_name] is the string constant's index in the const pool
    // The [val_reg] is the register where the value for the field is stored
    SetField(27,5) { sign_reg: u8, field_name: u16, val_reg: u8 },

    GetField(28, 5) { dest: u8, sign_reg: u8, field_name: u16 },

    // Deck operations.
    NewDeck(29, 6) { dest: u8, start_reg: u8, count: u8 },
    NewFixedDeck(30, 6) { dest: u8, start_reg: u8, count: u8, capacity: u16 },
    AddToDeck(31, 4) { deck: u8, position: u8, value: u8 },
    ExtractFromDeck(32, 4) { dest: u8, deck: u8, index: u8 },

}
