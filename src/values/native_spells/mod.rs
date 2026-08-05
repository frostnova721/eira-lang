pub mod deck;
pub mod io;
pub mod math;
pub mod num;
pub mod text;
pub mod time;

use crate::compiler::weaves::Weave;
use crate::values::spell::SpellInfo;
use crate::{EiraVM, Value};

pub type MethodInfoFn = fn(&Weave) -> Result<SpellInfo, String>;
pub type MethodHandlerFn = fn(&mut EiraVM, usize, usize) -> Result<Value, String>;

#[derive(Clone, Copy)]
pub struct NativeMethodDef {
    pub name: &'static str,
    pub spell_info: MethodInfoFn,
    pub handler: MethodHandlerFn,
}

pub type StandaloneInfoFn = fn() -> SpellInfo;
pub type StandaloneHandlerFn = fn(&mut EiraVM, usize, usize) -> Result<Value, String>;

#[derive(Clone, Copy)]
pub struct StandaloneSpellDef {
    pub name: &'static str,
    pub spell_info: StandaloneInfoFn,
    pub handler: StandaloneHandlerFn,
}

pub fn find_method(target_weave_name: &str, method_name: &str) -> Option<&'static NativeMethodDef> {
    let methods: &[NativeMethodDef] = match target_weave_name {
        "Deck" => deck::DECK_METHODS,
        "Text" => text::TEXT_METHODS,
        "Num" => num::NUM_METHODS,
        _ => return None,
    };
    methods.iter().find(|m| m.name == method_name)
}

pub static STANDALONE_SPELLS: &[StandaloneSpellDef] = &[
    StandaloneSpellDef {
        name: "listen",
        spell_info: io::listen_info,
        handler: io::listen_handler,
    },
    StandaloneSpellDef {
        name: "ask",
        spell_info: io::ask_info,
        handler: io::ask_handler,
    },
    StandaloneSpellDef {
        name: "floor",
        spell_info: math::floor_info,
        handler: math::floor_handler,
    },
    StandaloneSpellDef {
        name: "ceil",
        spell_info: math::ceil_info,
        handler: math::ceil_handler,
    },
];

pub fn find_standalone_spell(name: &str) -> Option<&'static StandaloneSpellDef> {
    STANDALONE_SPELLS.iter().find(|s| s.name == name)
}
