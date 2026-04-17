use eira::{
    EiraVM,
    compiler::compiler::{Compiler, CompilerOptions},
};

fn main() {
    // let start = Instant::now();
    let mut args = std::env::args().collect::<Vec<String>>();

    let mut compiler_options = CompilerOptions {
        print_tokens: false,
        print_ast: None,
        print_woven_ast: None,
        print_instructions: false,
        print_bytecode: false,
    };

    let mut i = 0;

    loop {
        if i >= args.len() {
            break;
        }
        let arg = &args[i];
        if arg.starts_with("--") {
            let arg = &arg.replace("--", "");
            if *arg == "ptkn".to_owned() {
                compiler_options.print_tokens = true;
            } else if arg.starts_with("past") {
                let verbosity = arg
                    .strip_prefix("past=")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
                compiler_options.print_ast = Some(verbosity);
            } else if arg.starts_with("pwast") {
                let verbosity = arg
                    .strip_prefix("pwast=")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
                compiler_options.print_woven_ast = Some(verbosity);
            } else if *arg == "pinst".to_owned() {
                compiler_options.print_instructions = true;
            } else if *arg == "pbc".to_owned() {
                compiler_options.print_bytecode = true;
            }
            args.remove(i);
        } else {
            i += 1;
        }
    }

    let default_debug_file = "tests/test.eira".to_string();
    let file_path = args.get(1).unwrap_or(&default_debug_file);

    let compiler = Compiler::new(file_path.clone(), compiler_options);

    let compiled = compiler.compile_to_bytecode();

    if compiled.is_err() {
        println!("The eira was cursed during the compilation of the scroll.");
        println!("{}", compiled.err().unwrap().msg);
        return;
    }

    EiraVM::init(compiled.ok().unwrap()).start();
}
