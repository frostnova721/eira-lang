use std::fs;

use eira::{CodeGen, EiraVM, Parser, Scanner, WeaveAnalyzer, print_byte_code, print_ast, print_woven_ast};

fn main() {
    // let start = Instant::now();
    let mut args = std::env::args().collect::<Vec<String>>();

    let mut flag_print_tokens = false;
    let mut flag_print_ast: Option<u8> = None;
    let mut flag_print_woven_ast: Option<u8> = None;
    let mut flag_print_instructions = false;
    let mut flag_print_bytecode = false;

    let mut i = 0;

    loop {
        if i >= args.len() {
            break;
        }
        let arg = &args[i];
        if arg.starts_with("--") {
            let arg = &arg.replace("--", "");
            if *arg == "ptkn".to_owned() {
                flag_print_tokens = true;
            } else if arg.starts_with("past") {
                let verbosity = arg.strip_prefix("past=").and_then(|v| v.parse().ok()).unwrap_or(0);
                flag_print_ast = Some(verbosity);
            } else if arg.starts_with("pwast") {
                let verbosity = arg.strip_prefix("pwast=").and_then(|v| v.parse().ok()).unwrap_or(0);
                flag_print_woven_ast = Some(verbosity);
            } else if *arg == "pinst".to_owned() {
                flag_print_instructions = true;
            } else if *arg == "pbc".to_owned() {
                flag_print_bytecode = true;
            }
            args.remove(i);
        } else {
            i += 1;
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
        for token in &tokens {
            println!("{}", token);
        }
    }

    // let scanTime = start.elapsed();

    let parser = Parser::new(tokens, file_path.to_string());
    let ast = parser.parse();

    if ast.is_err() {
        // println!("Parse Error: {:?}", ast.unwrap_err().0);
        return;
    }

    // let parseTime = start.elapsed();

    if let Some(verbosity) = flag_print_ast {
        println!("AST:");
        print_ast(ast.as_ref().unwrap(), verbosity);
    }

    let mut weave_analyzer = WeaveAnalyzer::new();
    let woven_tree = weave_analyzer.analyze(ast.unwrap());
    match woven_tree {
        Err(no_no) => {
            println!(
                "Weave Error: {}\nat '{}' in {}:{}:{}",
                no_no.msg, no_no.token.lexeme, file_path,no_no.token.line, no_no.token.column,
            )
        }
        Ok(yes_yes) => {
            if let Some(verbosity) = flag_print_woven_ast {
                println!("\nWoven Tree:");
                print_woven_ast(&yes_yes, verbosity);
            }

            let mut generator = CodeGen::new(yes_yes);
            generator.print_instructions = flag_print_instructions;
            let bc = generator.summon_bytecode().unwrap();

            if flag_print_bytecode {
                print_byte_code(&bc);
                println!("===DEBUG LOG END===\n\n");
            }
            let consts = generator.get_constants();
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
