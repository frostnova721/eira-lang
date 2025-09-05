use std::fs;

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

    println!("AST:");
    println!("{:?}", ast);

    let mut weave_analyzer = WeaveAnalyzer::new();
    let woven_tree = weave_analyzer.anaylze(ast);
    match woven_tree {
        Err(no_no) => {
            println!(
                "Weave Error: {}\nError at '{}' in line {}:{}",
                no_no.msg, no_no.token.lexeme, no_no.token.line, no_no.token.column,
            )
        }
        Ok(yes_yes) => {
            println!("\nWoven Tree:");
            println!("{:?}", yes_yes);
        }
    }
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
