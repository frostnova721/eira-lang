use crate::{
    EiraVM, Value,
    compiler::weaves::Weave,
    values::{native_spells::io::read_line, spell::SpellInfo},
};

#[derive(Debug, Clone, PartialEq)]
pub enum NativeSpell {
    Time,
    Math,
    Io(IoSpells),
}

impl NativeSpell {
    pub fn resolve(name: &str) -> Result<NativeSpell, String> {
        match name {
            "read_line" => Ok(NativeSpell::Io(IoSpells::ReadLine(SpellInfo {
                name: "read_line".to_string(),
                reagents: vec![],
                release_weave: Weave::Text,
                upvalues: vec![],
            }))),
            _ => Err(format!("Could'nt find a native spell for '{}'", name).to_string()),
        }
    }

    pub fn get_spell_info(spell: NativeSpell) -> Result<SpellInfo, String> {
        match spell {
            NativeSpell::Io(ios) => IoSpells::get_spell_info(ios),
            NativeSpell::Math => todo!("unimplemented"),
            NativeSpell::Time => todo!("unimplemented"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IoSpells {
    ReadLine(SpellInfo),
}

impl IoSpells {
    pub fn get_spell_info(spell: IoSpells) -> Result<SpellInfo, String> {
        return match spell {
            IoSpells::ReadLine(si) => Ok(si),
        }
    }
}

pub fn dispatch(_vm: &mut EiraVM, spell: NativeSpell, args: &[Value]) -> Result<Value, String> {
    match spell {
        NativeSpell::Time => Err("".to_string()),
        NativeSpell::Io(spells) => match spells {
            IoSpells::ReadLine(_) => read_line(),
        },
        NativeSpell::Math => Err("".to_string()),
    }
}
