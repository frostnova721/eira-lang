use std::rc::Rc;

use crate::{frontend::{reagents::WovenReagent, symbol_table::Symbol, weaves::Weave}, value::Value};

#[derive(Debug)]
pub struct SpellObject {
    pub name: Option<String>,
    pub arity: u8, // 255 should be enough. (please seek help if its not for you)
    pub upvalue_count: i32,
    pub constants: Vec<Value>,
    pub bytecode: Vec<u8>
    // asynchronous: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpellInfo {
    pub name: String,
    pub reagents: Vec<WovenReagent>,
    pub release_weave: Weave,
    pub symbol: Symbol,
}

#[derive(Debug)]
pub struct ClosureObject {
    pub spell: SpellObject,
    pub upvalues: Vec<Rc<UpValue>>
}

#[derive(Debug)]
pub struct UpValue {
    pub next: Option<Box<UpValue>>,
    pub location: *mut Value,
    pub closed: Value, 
}