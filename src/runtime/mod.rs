#[macro_use]
pub mod instruction_macro;

// Legacy modules (can be removed after migration)
// pub mod instruction;
// pub mod operation;

pub mod vm;

// Re-export the macro-generated types
pub use instruction_macro::{Instruction, OpCode};