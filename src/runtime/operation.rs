use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
pub enum OpCode {
    // Arithematic
    Add,
    Subtract,
    Divide,
    Multiply,

    // Comparison
    Equal,
    Greater,
    Less,

    // bool
    False,
    True,

    // Bitwise
    Negate,
    Not,

    // Constants/Values
    Constant,

    // Concat
    Concat,

    // idk
    Print,

    // halt!
    Halt,

    // Globals
    SetGlobal,
    GetGlobal,

    // Locals
    // SetLocal,
    // GetLocal,
    Move, // move value between local slots

    // Empty value (like null, but not null)
    Emptiness,

    // Pop from locals stack
    PopStack,

    // Jump statements
    Jump,
    JumpIfFalse,

    //loop
    Loop,

    // function calls
    Cast,

    Release,
}

impl OpCode {
    pub fn to_debug_string(&self) -> String {
        match self {
            OpCode::Add => "OP_ADD",
            OpCode::Constant => "OP_CONST",
            OpCode::Divide => "OP_DIV",
            OpCode::Multiply => "OP_MUL",
            OpCode::Negate => "OP_NEG",
            OpCode::Not => "OP_NOT",
            OpCode::Concat => "OP_CONCAT",
            OpCode::Print => "OP_PRINT",
            OpCode::Subtract => "OP_SUB",
            OpCode::Equal => "OP_EQUAL",
            OpCode::Greater => "OP_GREATER",
            OpCode::Less => "OP_LESS",
            OpCode::GetGlobal => "OP_GET_GLOBAL",
            OpCode::SetGlobal => "OP_SET_GLOBAL",
            // OpCode::SetLocal => "OP_SET_LOCAL",
            // OpCode::GetLocal => "OP_GET_LOCAL",
            OpCode::Move => "OP_MOVE",
            OpCode::Emptiness => "OP_EMPTINESS",
            OpCode::PopStack => "OP_POP_STACK",
            OpCode::Jump => "OP_JUMP",
            OpCode::JumpIfFalse => "OP_JUMP_FALSE",
            OpCode::Loop => "OP_LOOP",
            OpCode::Release => "OP_RETURN",
            OpCode::True => "OP_TRUE",
            OpCode::False => "OP_FALSE",
            OpCode::Cast => "OP_CAST",
            _ => "OP_UNKNOWN",
        }
        .to_owned()
    }

    pub fn inst_len(&self) -> usize {
        match self {            
            OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Equal
            | OpCode::Less
            | OpCode::Constant
            | OpCode::SetGlobal
            | OpCode::GetGlobal
            // | OpCode::SetLocal
            // | OpCode::GetLocal
            | OpCode::Move
            | OpCode::JumpIfFalse
            | OpCode::Concat
            | OpCode::Cast
            | OpCode::Greater => 4, // opcode + dest + r1 + r2

            OpCode::Negate | OpCode::Not | OpCode::PopStack | OpCode::Jump | OpCode::Loop => 3, // opcode + dest + r1

            OpCode::True | OpCode::False | OpCode::Print | OpCode::Emptiness | OpCode::Release => 2, // opcode + r1/dest

            OpCode::Halt => 1, // just the opcode
        }
    }
}
