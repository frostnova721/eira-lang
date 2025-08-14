use crate::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct SpellObject {
    pub name: String,
    pub arity: u8, // 255 should be enough. (please seek help if its not for you)
    pub instructions: Vec<Instruction>,
    pub upvalue_count: i32,
    // asynchronous: bool,
}
