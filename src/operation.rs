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

    Halt,
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
            | OpCode::Greater => 4, // opcode + dest + r1 + r2

            OpCode::Negate | OpCode::Not => 3, // opcode + dest + r1

            OpCode::True | OpCode::False | OpCode::Print => 2, // opcode + r1/dest

            OpCode::Halt => 1, // just the opcode
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

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
            _ => Err(()),
        }
    }
}
