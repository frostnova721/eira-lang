use std::fs;

use eira::{CodeGen, EiraVM, Parser, Scanner, WeaveAnalyzer, print_byte_code};

fn main() {
    // let start = Instant::now();
    let mut args = std::env::args().collect::<Vec<String>>();

    let mut flag_print_tokens = false;
    let mut flag_print_ast = false;
    let mut flag_print_woven_ast = false;
    let mut flag_print_instructions = false;
    let mut flag_print_bytecode = false;

    for i in 0..args.len() {
        let arg = &args[i];
        if arg.starts_with("--") {
            let arg = &arg.replace("--", "");
            if *arg == "ptkn".to_owned() {
                flag_print_tokens = true;
            }
            if *arg == "past".to_owned() {
                flag_print_ast = true;
            } else if *arg == "pwast".to_owned() {
                flag_print_woven_ast = true;
            } else if *arg == "pinst".to_owned() {
                flag_print_instructions = true;
            } else if *arg == "pbc".to_owned() {
                flag_print_bytecode = true;
            }
            args.remove(i);
        }
    }

    let default_debug_file = "tests/test.eira".to_string();
    let file_path = args.get(1).unwrap_or(&default_debug_file);

    let f = fs::read_to_string(file_path);
    let binding = f.unwrap();
    let scanner = Scanner::init(&binding);
    let tokens = scanner.tokenize();

    if flag_print_tokens {
        println!("Tokens:");
        println!("{:?}", tokens)
    }

    // let scanTime = start.elapsed();

    let parser = Parser::new(tokens);
    let ast = parser.parse();

    if ast.is_err() {
        println!("Parse Error: {:?}", ast.unwrap_err());
        return;
    }

    // let parseTime = start.elapsed();

    if flag_print_ast {
        println!("AST:");
        println!("{:?}", ast.as_ref().unwrap());
    }

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
            if flag_print_woven_ast {
                println!("\nWoven Tree:");
                println!("{:?}", yes_yes);
            }

            let mut generator = CodeGen::new(yes_yes);
            let bc = generator.summon_bytecode().unwrap();

            if flag_print_bytecode {
                print_byte_code(&bc);
            }
            let consts = generator.get_constants();
            println!("===DEBUG LOG END===\n\n");
            // println!("{:?}", consts);
            EiraVM::init(bc, consts).start();
        }
    }
    // println!(
    //     "\nTime taken: Scan: {:?}, Parse: {:?}, Total: {:?}",
    //     scanTime,
    //     parseTime - scanTime,
    //     start.elapsed()
    // );
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
