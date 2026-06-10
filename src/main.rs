use std::path::Path;

use eira::{
    EiraVM,
    compiler::compiler::{Compiler, CompilerOptions},
    project::config::Project,
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

    let project_root = Project::find_root(Path::new(
        &args.get(1).cloned().unwrap_or_else(|| ".".to_string()),
    ));

    let project = if let Some(root) = project_root {
        Project::load_from_toml(root.as_path().to_str().unwrap_or("essence.toml")).ok()
    } else {
        None
    };

    let target_file_path = if let Some(path) = args.get(1) {
        path.clone()
    } else if let Some(proj) = &project {
        proj.entry_point.clone()
    } else {
        eprintln!("No source file provided and no project configuration found.");
        return;
    };

    let compiler = Compiler::new(target_file_path.clone(), compiler_options, project);

    let compiled = compiler.compile_to_bytecode();

    if compiled.is_err() {
        eprintln!("The eira was cursed during the compilation of the scroll.");
        eprintln!("{}", compiled.err().unwrap().msg);
        return;
    }

    EiraVM::init(compiled.ok().unwrap()).start();
}
