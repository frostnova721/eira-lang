#[macro_use]
pub mod instruction_macro;

pub mod vm;

// Re-export the macro-generated types
pub use instruction_macro::{Instruction, OpCode};
