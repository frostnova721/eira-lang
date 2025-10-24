use std::{fs};

use eira::{print_byte_code, CodeGen, EiraVM, Parser, Scanner, WeaveAnalyzer};

fn main() {
    let f = fs::read_to_string("tests/test.eira");
    let binding = f.unwrap();
    let scanner = Scanner::init(&binding);
    let tokens = scanner.tokenize();
    let parser = Parser::new(tokens);
    let ast = parser.parse();

    if ast.is_err() {
        println!("Parse Error: {:?}", ast.err().unwrap());
        return;
    }

    // println!("AST:");
    // println!("{:?}", ast);

    let mut weave_analyzer = WeaveAnalyzer::new();
    let woven_tree = weave_analyzer.analyze(ast.unwrap());
    match woven_tree {
        Err(no_no) => {
            println!(
                "Weave Error: {}\nError at '{}' in line {}:{}",
                no_no.msg, no_no.token.lexeme, no_no.token.line, no_no.token.column,
            )
        }
        Ok(yes_yes) => {
            // println!("\nWoven Tree:");
            // println!("{:?}", yes_yes);

            let mut generator = CodeGen::new(yes_yes);
            let bc = generator.summon_bytecode().unwrap();
            print_byte_code(&bc);
            let consts =  generator.get_constants();
            println!("===DEBUG LOG END===\n\n");
            // println!("{:?}", consts);
            EiraVM::init(bc, consts).start();
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