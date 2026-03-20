pub mod assembler;
pub mod ast_printer;
pub mod debug;
pub mod compiler;
pub mod runtime;
pub mod values;

pub use compiler::scanner::{Scanner, Token};
pub use compiler::parser::{Parser};
pub use compiler::weave_analyser::{WeaveAnalyzer};
pub use compiler::code_gen::{CodeGen};

pub use values::value::{Value, ValueType};
pub use values::spell::{SpellObject, ClosureObject};

pub use runtime::vm::EiraVM;

pub use debug::{print_byte_code, print_instructions};
pub use ast_printer::{print_ast, print_woven_ast};