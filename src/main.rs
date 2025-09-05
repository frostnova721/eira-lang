use std::{
    fs,
};

use crate::frontend::{parser::Parser, scanner::Scanner, weave_analyser::WeaveAnalyzer};


mod assembler;
mod debug;
mod frontend;
mod runtime;

fn main() {
    let f = fs::read_to_string("tests/test.eira");
    let binding = f.unwrap();
    let scanner = Scanner::init(&binding);
    let tokens = scanner.tokenize();
    let parser = Parser::new(tokens);
    let ast = parser.parse();
    
    println!("{:?}", ast);

    let mut weave_analyzer = WeaveAnalyzer::new();
    weave_analyzer.anaylze(ast);
    // let mut compiler = Compiler::init_compiler(binding.as_str());
    // let instructions = compiler.compile();
    // match instructions {
    //     Ok(inst) => {
    //         println!("Compile OK.");
    //         let constants = compiler.constants;
    //         print_instructions(inst.clone(), &constants);
    //         let bc = Assembler::convert_to_byte_code(inst);
    //         print_byte_code(bc.clone());
    //         let mut vm = EiraVM::init(bc, constants);
    //         vm.start();
    //     }
    //     Err(_) => {
    //         // println!("Compile Error: {}", e);
    //         return;
    //     }
    // }
}
