use std::fs;

use crate::{
    compiler::Compiler,
    debug::{print_byte_code, print_instructions},
    disassembler::Disassembler,
    vm::EiraVM,
};

mod chunk;
mod compiler;
mod debug;
mod disassembler;
mod instruction;
mod operation;
mod scanner;
mod token_type;
mod value;
mod vm;

fn main() {
    let f = fs::read_to_string("tests/test.eira");
    let binding = f.unwrap();
    let mut compiler = Compiler::init_compiler(binding.as_str());
    let instructions = compiler.compile();
    match instructions {
        Ok(inst) => {
            println!("Compile OK.");
            let constants = compiler.constants.clone();
            print_instructions(inst.clone(), constants.clone());
            let bc = Disassembler::convert_to_byte_code(inst);
            print_byte_code(bc.clone());
            let mut vm = EiraVM::init(bc, constants);
            vm.start();
        }
        Err(e) => {
            // println!("Compile Error: {}", e);
            return;
        }
    }
}
