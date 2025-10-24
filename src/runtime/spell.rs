use std::rc::Rc;

use crate::{frontend::{reagents::WovenReagent, symbol_table::Symbol, weaves::Weave}, value::Value};

#[derive(Debug, Clone)]
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
    pub release_weave: Weave, // weave expected to be released
    pub released_weave: Option<Weave>, // the actual weave that was released
    pub symbol: Symbol,
    pub upvalues: Vec<UpValue>,
}

#[derive(Debug)]
pub struct ClosureObject {
    pub spell: Rc<SpellObject>,
    pub upvalues: Vec<UpValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpValue {
    pub index: usize, // will contain the absolute slot index!
    pub closed: Value, 
}