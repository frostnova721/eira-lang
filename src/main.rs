use std::fs;

use crate::{compiler::Compiler, debug::{print_byte_code, print_instructions}, disassembler::Disassembler, vm::EiraVM};

mod vm;
mod compiler;
mod scanner;
mod token_type;
mod chunk;
mod operation;
mod debug;
mod value;
mod instruction;
mod disassembler;

fn main() {
    let f = fs::read_to_string("tests/test.eira");
    let binding = f.unwrap();
    let mut compiler = Compiler::init_compiler(binding.as_str());
    let instructions = compiler.compile();
    let constants = compiler.constants.clone();
    print_instructions(instructions.clone(), constants.clone());
    let bc = Disassembler::convert_to_byte_code(instructions);
    print_byte_code(bc.clone());
    let mut vm = EiraVM::init(bc, constants);
    vm.start();
}
