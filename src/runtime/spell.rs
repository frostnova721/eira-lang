use std::rc::Rc;

use crate::{runtime::spell, value::Value};

#[derive(Debug)]
pub struct SpellObject {
    pub name: Option<&'static str>,
    pub arity: u8, // 255 should be enough. (please seek help if its not for you)
    pub upvalue_count: i32,
    pub constants: Vec<Value>,
    pub bytecode: Vec<u8>
    // asynchronous: bool,
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