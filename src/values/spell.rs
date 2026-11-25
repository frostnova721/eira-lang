use std::{cell::RefCell, rc::Rc};

use crate::{
    compiler::{reagents::WovenReagent, weaves::Weave},
    values::value::Value,
};

#[derive(Debug, Clone)]
pub struct SpellObject {
    pub name: Option<String>,
    pub arity: u8, // 255 should be enough. (please seek help if its not for you)
    pub upvalue_count: i32,
    pub constants: Vec<Value>,
    pub bytecode: Vec<u8>, // asynchronous: bool,
    /// Maps bytecode offsets back to source locations, for runtime errors.
    pub lines: LineTable,
}

/// A source location attached to a run of bytecode.
#[derive(Debug, Clone, PartialEq)]
pub struct LineEntry {
    /// Offset of the first byte this entry covers.
    pub offset: usize,
    pub file: Rc<str>,
    pub line: usize,
    pub column: usize,
}

/// Sorted by offset. An entry covers bytes up to the next entry's offset.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LineTable {
    pub entries: Vec<LineEntry>,
}

impl LineTable {
    /// Finds the location of the instruction containing the byte at `offset`.
    pub fn lookup(&self, offset: usize) -> Option<&LineEntry> {
        let idx = self.entries.partition_point(|e| e.offset <= offset);
        if idx == 0 {
            None
        } else {
            self.entries.get(idx - 1)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpellInfo {
    /// The name of the spell
    pub name: String,

    /// The reagents(arguments) required by the spell
    pub reagents: Vec<WovenReagent>,

    /// The weave expected to be released
    pub release_weave: Weave,

    /// The actual weave that was released
    // pub released_weave: Option<Weave>,

    /// The representation for symbol table
    // pub symbol: Symbol,

    /// The symbol representing the released weave (How about a gamble?)
    // pub released_symbol: Option<Symbol>,

    /// The upvalues captured by the spell
    pub upvalues: Vec<UpValue>,
}

#[derive(Debug, Clone)]
pub struct ClosureObject {
    pub spell: Rc<SpellObject>,
    pub upvalues: Vec<UpValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpValue {
    pub index: usize, // will contain the absolute slot index!
    pub depth: usize,
    pub closed: RefCell<Value>,
}
