#[repr(u8)]
#[derive(Debug)]
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

    // idk
    Print,

    // halt!
    Halt,

    // Globals
    SetGlobal,
    GetGlobal,

    // Locals
    SetLocal,
    GetLocal,

    // Empty value (like null, but not null)
    Emptiness,

    // Pop from locals stack
    PopStack,

    // Jump statements
    Jump,
    JumpIfFalse,

    //loop
    Loop,
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
            OpCode::Print => "OP_PRINT",
            OpCode::Subtract => "OP_SUB",
            OpCode::Equal => "OP_EQUAL",
            OpCode::Greater => "OP_GREATER",
            OpCode::Less => "OP_LESS",
            OpCode::GetGlobal => "OP_GET_GLOBAL",
            OpCode::SetGlobal => "OP_SET_GLOBAL",
            OpCode::SetLocal => "OP_SET_LOCAL",
            OpCode::GetLocal => "OP_GET_LOCAL",
            OpCode::Emptiness => "OP_EMPTINESS",
            OpCode::PopStack => "OP_POP_STACK",
            OpCode::Jump => "OP_JUMP",
            OpCode::JumpIfFalse => "OP_JUMP_FALSE",
            OpCode::Loop => "OP_LOOP",
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
            | OpCode::SetLocal
            | OpCode::GetLocal
            | OpCode::JumpIfFalse
            | OpCode::Greater => 4, // opcode + dest + r1 + r2

            OpCode::Negate 
            | OpCode::Not 
            | OpCode::PopStack 
            | OpCode::Jump 
            | OpCode::Loop => 3, // opcode + dest + r1

            OpCode::True 
            | OpCode::False 
            | OpCode::Print 
            | OpCode::Emptiness => 2, // opcode + r1/dest

            OpCode::Halt => 1, // just the opcode
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    // Order messed up = "kaboom RICO!"
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Add),
            1 => Ok(OpCode::Subtract),
            2 => Ok(OpCode::Divide),
            3 => Ok(OpCode::Multiply),
            4 => Ok(OpCode::Equal),
            5 => Ok(OpCode::Greater),
            6 => Ok(OpCode::Less),
            7 => Ok(OpCode::False),
            8 => Ok(OpCode::True),
            9 => Ok(OpCode::Negate),
            10 => Ok(OpCode::Not),
            11 => Ok(OpCode::Constant),
            12 => Ok(OpCode::Print),
            13 => Ok(OpCode::Halt),
            14 => Ok(OpCode::SetGlobal),
            15 => Ok(OpCode::GetGlobal),
            16 => Ok(OpCode::SetLocal),
            17 => Ok(OpCode::GetLocal),
            18 => Ok(OpCode::Emptiness),
            19 => Ok(OpCode::PopStack),
            20 => Ok(OpCode::Jump),
            21 => Ok(OpCode::JumpIfFalse),
            22 => Ok(OpCode::Loop),

            _ => Err(()),
        }
    }
}
