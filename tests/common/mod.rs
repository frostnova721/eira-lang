use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use eira::compiler::compiler::{Compiler, CompilerOptions};

/// Each test owns an isolated directory, including when Cargo runs tests in parallel.
pub struct SourceProject {
    pub root: PathBuf,
}

impl SourceProject {
    pub fn new(source: &str) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "eira-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create test directory");
        let project = Self { root };
        project.write("main.eira", source);
        project
    }

    pub fn write(&self, name: &str, source: &str) {
        fs::write(self.root.join(name), source).expect("write test source");
    }

    pub fn compiler(&self) -> Compiler {
        Compiler::new(
            self.root.join("main.eira").to_str().unwrap().to_owned(),
            CompilerOptions {
                print_tokens: false,
                print_ast: None,
                print_woven_ast: None,
                print_instructions: false,
                print_bytecode: false,
            },
            None,
        )
    }
}

impl Drop for SourceProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
