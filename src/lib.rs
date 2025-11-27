pub mod assembler;
pub mod debug;
pub mod frontend;
pub mod runtime;
pub mod values;

pub use frontend::scanner::{Scanner, Token};
pub use frontend::parser::{Parser};
pub use frontend::weave_analyser::{WeaveAnalyzer};
pub use frontend::code_gen::{CodeGen};

pub use values::value::{Value, ValueType};
pub use values::spell::{SpellObject, ClosureObject};

pub use runtime::vm::EiraVM;

pub use debug::{print_byte_code, print_instructions};