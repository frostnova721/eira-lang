pub mod assembler;
pub mod ast_printer;
pub mod compiler;
pub mod debug;
pub mod logger;
pub mod runtime;
pub mod values;

pub use compiler::code_gen::CodeGen;
pub use compiler::parser::Parser;
pub use compiler::scanner::{Scanner, Token};
pub use compiler::weave_analyser::WeaveAnalyzer;

pub use values::spell::{ClosureObject, SpellObject};
pub use values::value::{Value, ValueType};

pub use runtime::vm::EiraVM;

pub use ast_printer::{print_ast, print_woven_ast};
pub use debug::{print_byte_code, print_instructions};
