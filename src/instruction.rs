use crate::operation::OpCode;

#[derive(Clone, Copy)]
pub enum Instruction {
    Add { dest: u8, r1: u8, r2: u8 },
    Subtract { dest: u8, r1: u8, r2: u8 },
    Multiply { dest: u8, r1: u8, r2: u8 },
    Divide { dest: u8, r1: u8, r2: u8 },

    Equal { dest: u8, r1: u8, r2: u8 },
    Greater { dest: u8, r1: u8, r2: u8 },
    Less { dest: u8, r1: u8, r2: u8 },

    Negate { dest: u8, r1: u8 },
    Not { dest: u8, r1: u8 },

    Constant { dest: u8, const_index: u16 },
    True { dest: u8 },
    False { dest: u8 },
    Print { r1: u8 },

    Halt,
}

impl Instruction {
    /// Returns the length of the instruction
    pub fn len(self) -> usize {
        match self {
            Instruction::Add { dest: _, r1: _, r2: _ } => 4,
            Instruction::Subtract { dest:_, r1:_, r2:_ } => 4,
            Instruction::Multiply { dest:_, r1:_, r2:_ } => 4,
            Instruction::Divide { dest:_, r1:_, r2:_ } => 4,
            Instruction::Equal { dest:_, r1:_, r2:_ } => 4,
            Instruction::Greater { dest:_, r1:_, r2:_ } => 4,
            Instruction::Less { dest:_, r1:_, r2:_ } => 4,
            Instruction::Constant { dest:_, const_index:_ } => 4, // 4 since u16 -> 2 bytes
            Instruction::Negate { dest:_, r1:_ } => 3,
            Instruction::Not { dest:_, r1:_ } => 3,
            Instruction::True { dest:_ } => 2,
            Instruction::False { dest:_ } => 2,
            Instruction::Print { r1:_ } => 2,
            Instruction::Halt => 1,
        }
    }

    /// Returns the string representation of the [Instruction]
    pub fn to_string(self) -> String {
        match self {
            Instruction::Add { dest, r1, r2 } => format!("ADD {} {} {}", dest, r1, r2),
            Instruction::Subtract { dest, r1, r2 } => format!("SUBTRACT {} {} {}", dest, r1, r2),
            Instruction::Multiply { dest, r1, r2 } => format!("MULTIPLY {} {} {}", dest, r1, r2),
            Instruction::Divide { dest, r1, r2 } => format!("DIVIDE {} {} {}", dest, r1, r2),
            Instruction::Constant { dest, const_index } => {
                format!("CONSTANT {} {}", dest, const_index)
            }
            Instruction::Negate { dest, r1 } => format!("NEGATE {} {}", dest, r1),
            Instruction::Not { dest, r1 } => format!("NOT {} {}", dest, r1),
            Instruction::True { dest } => format!("TRUE {}", dest),
            Instruction::False { dest } => format!("FALSE {}", dest),
            Instruction::Print { r1 } => format!("PRINT {}", r1),
            Instruction::Equal { dest, r1, r2 } => format!("EQUAL {} {} {}", dest, r1, r2),
            Instruction::Greater { dest, r1, r2 } => format!("GREATER {} {} {}", dest, r1, r2),
            Instruction::Less { dest, r1, r2 } => format!("LESS {} {} {}", dest, r1, r2),
            Instruction::Halt => "Halt".to_owned(),
        }
    }

    pub fn get_byte_code(&self) -> Vec<u8> {
        match self {
            Instruction::Add { dest, r1, r2 } => vec![OpCode::Add as u8, *dest, *r1, *r2],
            Instruction::Subtract { dest, r1, r2 } => vec![OpCode::Subtract as u8, *dest, *r1, *r2],
            Instruction::Multiply { dest, r1, r2 } => vec![OpCode::Multiply as u8, *dest, *r1, *r2],
            Instruction::Divide { dest, r1, r2 } => vec![OpCode::Divide as u8, *dest, *r1, *r2],
            Instruction::Constant { dest, const_index } => {
                let index_bytes = (*const_index).to_le_bytes(); // let's say index is u16
                let mut bytes = vec![OpCode::Constant as u8, *dest];
                bytes.extend_from_slice(&index_bytes);
                bytes
            }
            Instruction::Negate { dest, r1 } => vec![OpCode::Negate as u8, *dest, *r1],
            Instruction::Not { dest, r1 } => vec![OpCode::Not as u8, *dest, *r1],
            Instruction::True { dest } => vec![OpCode::True as u8, *dest],
            Instruction::False { dest } => vec![OpCode::False as u8, *dest],
            Instruction::Greater { dest, r1, r2 } => vec![OpCode::Greater as u8, *dest, *r1, *r2],
            Instruction::Less { dest, r1, r2 } => vec![OpCode::Less as u8, *dest, *r1, *r2],
            Instruction::Equal { dest, r1, r2 } => vec![OpCode::Equal as u8, *dest, *r1, *r2],
            Instruction::Print { r1 } => vec![OpCode::Print as u8, *r1],
            Instruction::Halt => vec![OpCode::Halt as u8],
        }
    }
}
