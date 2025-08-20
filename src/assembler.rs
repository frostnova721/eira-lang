use crate::runtime::instruction::{self, Instruction};

pub struct Assembler {}

impl Assembler {
    pub fn convert_to_byte_code(instructions: Vec<Instruction>) -> Vec<u8> {
        let mut bc: Vec<u8> = vec![];
        for inst in instructions {
            let instruction_bytes = inst.get_byte_code();
            bc.extend(instruction_bytes);
        }
        bc
    }
}
