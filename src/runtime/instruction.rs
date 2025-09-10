use std::{fmt::format, vec};

use crate::runtime::operation::OpCode;

#[derive(Debug, Clone, Copy)]
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

    GetGlobal { dest: u8, const_index: u16 },
    SetGlobal { src_reg: u8, const_index: u16 },

    GetLocal { dest: u8, slot_index: u16 },
    SetLocal { src_reg: u8, slot_idx: u16 },

    True { dest: u8 },
    False { dest: u8 },
    Print { r1: u8 },

    Emptiness { dest: u8 },

    PopStack { pop_count: u16 },

    Jump { offset: u16 },
    JumpIfFalse { condition_reg: u8, offset: u16 },

    Loop { offset: u16 },

    Concat { dest: u8, r1: u8, r2: u8 },

    Halt,
}

impl Instruction {
    /// Returns the length of the instruction
    pub fn len(self) -> usize {
        match self {
            Instruction::Add {
                dest: _,
                r1: _,
                r2: _,
            } => 4,
            Instruction::Subtract {
                dest: _,
                r1: _,
                r2: _,
            } => 4,
            Instruction::Multiply {
                dest: _,
                r1: _,
                r2: _,
            } => 4,
            Instruction::Divide {
                dest: _,
                r1: _,
                r2: _,
            } => 4,
            Instruction::Equal {
                dest: _,
                r1: _,
                r2: _,
            } => 4,
            Instruction::Greater {
                dest: _,
                r1: _,
                r2: _,
            } => 4,
            Instruction::Less {
                dest: _,
                r1: _,
                r2: _,
            } => 4,
            Instruction::GetGlobal {
                dest: _,
                const_index: _,
            } => 4,
            Instruction::SetGlobal {
                src_reg: _,
                const_index: _,
            } => 4,
            Instruction::GetLocal {
                dest: _,
                slot_index: _,
            } => 4,
            Instruction::SetLocal {
                src_reg: _,
                slot_idx: _,
            } => 4,
            Instruction::Constant {
                dest: _,
                const_index: _,
            } => 4, // 4 since u16 -> 2 bytes
            Instruction::JumpIfFalse {
                condition_reg: _,
                offset: _,
            } => 4,
            Instruction::Concat { dest, r1, r2 } => 4,
            Instruction::Negate { dest: _, r1: _ } => 3,
            Instruction::Not { dest: _, r1: _ } => 3,
            Instruction::Jump { offset: _ } => 3,
            Instruction::Loop { offset:_ } => 3,
            Instruction::PopStack { pop_count: _ } => 3,
            Instruction::True { dest: _ } => 2,
            Instruction::False { dest: _ } => 2,
            Instruction::Print { r1: _ } => 2,
            Instruction::Emptiness { dest: _ } => 2,
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
            Instruction::GetGlobal { dest, const_index } => {
                format!("GET_GLOBAL {} {}", dest, const_index)
            }
            Instruction::SetGlobal {
                src_reg,
                const_index,
            } => format!("SET_GLOBAL {} {}", src_reg, const_index),
            Instruction::GetLocal { dest, slot_index } => {
                format!("GET_LOCAL {} {}", dest, slot_index)
            }
            Instruction::SetLocal { src_reg, slot_idx } => {
                format!("SET_LOCAL {} {}", src_reg, slot_idx)
            }
            Instruction::Negate { dest, r1 } => format!("NEGATE {} {}", dest, r1),
            Instruction::Not { dest, r1 } => format!("NOT {} {}", dest, r1),
            Instruction::True { dest } => format!("TRUE {}", dest),
            Instruction::False { dest } => format!("FALSE {}", dest),
            Instruction::Print { r1 } => format!("PRINT {}", r1),
            Instruction::Equal { dest, r1, r2 } => format!("EQUAL {} {} {}", dest, r1, r2),
            Instruction::Greater { dest, r1, r2 } => format!("GREATER {} {} {}", dest, r1, r2),
            Instruction::Less { dest, r1, r2 } => format!("LESS {} {} {}", dest, r1, r2),
            Instruction::Emptiness { dest } => format!("EMPTINESS {}", dest),
            Instruction::PopStack { pop_count } => format!("POPSTACK {}", pop_count),
            Instruction::Jump { offset } => format!("JUMP {}", offset),
            Instruction::JumpIfFalse { condition_reg, offset } => format!("JUMP_IF_FALSE {} {}", condition_reg, offset),
            Instruction::Loop { offset } => format!("LOOP {}", offset),
            Instruction::Concat { dest, r1, r2 } => format!("CONCAT {} {} {}", dest, r1, r2),
            Instruction::Halt => "Halt".to_owned(),
        }
    }

    /// Returns the bytecode for the specific instruction
    pub fn get_byte_code(&self) -> Vec<u8> {
        match self {
            Instruction::Add { dest, r1, r2 } => vec![OpCode::Add as u8, *dest, *r1, *r2],
            Instruction::Subtract { dest, r1, r2 } => vec![OpCode::Subtract as u8, *dest, *r1, *r2],
            Instruction::Multiply { dest, r1, r2 } => vec![OpCode::Multiply as u8, *dest, *r1, *r2],
            Instruction::Divide { dest, r1, r2 } => vec![OpCode::Divide as u8, *dest, *r1, *r2],
            Instruction::Constant { dest, const_index } => {
                self.gen_const_byte_code(OpCode::Constant, const_index, dest)
            }
            Instruction::GetGlobal { dest, const_index } => {
                self.gen_const_byte_code(OpCode::GetGlobal, const_index, dest)
            }
            Instruction::SetGlobal {
                src_reg,
                const_index,
            } => self.gen_const_byte_code(OpCode::SetGlobal, const_index, src_reg),
            Instruction::SetLocal { src_reg, slot_idx } => {
                self.gen_const_byte_code(OpCode::SetLocal, slot_idx, src_reg)
            }
            Instruction::GetLocal { dest, slot_index } => {
                self.gen_const_byte_code(OpCode::GetLocal, slot_index, dest)
            }
            Instruction::Negate { dest, r1 } => vec![OpCode::Negate as u8, *dest, *r1],
            Instruction::Not { dest, r1 } => vec![OpCode::Not as u8, *dest, *r1],
            Instruction::True { dest } => vec![OpCode::True as u8, *dest],
            Instruction::False { dest } => vec![OpCode::False as u8, *dest],
            Instruction::Greater { dest, r1, r2 } => vec![OpCode::Greater as u8, *dest, *r1, *r2],
            Instruction::Less { dest, r1, r2 } => vec![OpCode::Less as u8, *dest, *r1, *r2],
            Instruction::Equal { dest, r1, r2 } => vec![OpCode::Equal as u8, *dest, *r1, *r2],
            Instruction::Print { r1 } => vec![OpCode::Print as u8, *r1],
            Instruction::Emptiness { dest } => vec![OpCode::Emptiness as u8, *dest],
            Instruction::PopStack { pop_count } => {
                let (a, b) = self.split_u16(pop_count);
                vec![OpCode::PopStack as u8, a, b]
            }
            Instruction::Jump { offset } => {
                let (a,b) = self.split_u16(offset);
                vec![OpCode::Jump as u8, a,b]
            }
            Instruction::JumpIfFalse { condition_reg, offset } => {
                let (a,b) = self.split_u16(offset);
                vec![OpCode::JumpIfFalse as u8, *condition_reg, a, b]
            }
            Instruction::Loop { offset } => {
                let (a,b) = self.split_u16(offset);
                vec![OpCode::Loop as u8, a, b]
            }
            Instruction::Concat { dest, r1, r2 } => vec![OpCode::Concat as u8, *dest, *r1, *r2],
            Instruction::Halt => vec![OpCode::Halt as u8],
        }
    }

    fn gen_const_byte_code(&self, opcode: OpCode, const_index: &u16, src_reg: &u8) -> Vec<u8> {
        let index_bytes = (*const_index).to_le_bytes(); // let's say index is u16
        let mut bytes = vec![opcode as u8, *src_reg];
        bytes.extend_from_slice(&index_bytes);
        bytes
    }

    fn split_u16(&self, u16_: &u16) -> (u8, u8) {
        let bytes = (*u16_).to_le_bytes();
        (bytes[0], bytes[1])
    }
}
