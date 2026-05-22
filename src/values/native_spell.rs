use crate::{
    EiraVM, Value,
    compiler::{weaves::Weave, reagents::WovenReagent},
    values::{native_spells::io::read_line, spell::SpellInfo},
};

#[derive(Debug, Clone, PartialEq)]
pub enum NativeSpell {
    Time(TimeSpells),
    Math(MathSpells),
    Io(IoSpells),
}

impl NativeSpell {
    pub fn resolve(name: &str) -> Result<NativeSpell, String> {
        match name {
            "listen" => Ok(NativeSpell::Io(IoSpells::Listen(SpellInfo {
                name: "listen".to_string(),
                reagents: vec![],
                release_weave: Weave::Text,
                upvalues: vec![],
            }))),
            "ask" => Ok(NativeSpell::Io(IoSpells::Ask(SpellInfo {
                name: "ask".to_string(),
                reagents: vec![WovenReagent {
                    weave: Weave::Text,
                }],
                release_weave: Weave::Text,
                upvalues: vec![],
            }))),
            _ => Err(format!("Could'nt find a native spell for '{}'", name).to_string()),
        }
    }

    pub fn get_spell_info(spell: NativeSpell) -> Result<SpellInfo, String> {
        match spell {
            NativeSpell::Io(ios) => IoSpells::get_spell_info(ios),
            NativeSpell::Math(math) => MathSpells::get_spell_info(math),
            NativeSpell::Time(time) => TimeSpells::get_spell_info(time),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IoSpells {
    Listen(SpellInfo),
    Ask(SpellInfo),
}

impl IoSpells {
    pub fn get_spell_info(spell: IoSpells) -> Result<SpellInfo, String> {
        return match spell {
            IoSpells::Listen(si) => Ok(si),
            IoSpells::Ask(si) => Ok(si),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeSpells {}

impl TimeSpells {
    pub fn get_spell_info(_spell: TimeSpells) -> Result<SpellInfo, String> {
        todo!("yet to be implemented")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MathSpells {}

impl MathSpells {
    pub fn get_spell_info(_spell: MathSpells) -> Result<SpellInfo, String> {
        todo!("yet to be implemented")
    }
}

// dispatcher fucntion for native spells
pub fn dispatch(
    _vm: &mut EiraVM,
    spell: NativeSpell,
    arg_start_idx: usize,
    _argc: usize,
) -> Result<Value, String> {
    match spell {
        NativeSpell::Time(_spells) => todo!("yet to be implemented"),
        NativeSpell::Io(spells) => match spells {
            IoSpells::Listen(_) => read_line(None),
            IoSpells::Ask(_) => {
                let prompt_val = _vm.stack[arg_start_idx].clone();
                let prompt_str = prompt_val.extract_string().unwrap();
                read_line(Some(&prompt_str))
            }
        },
        NativeSpell::Math(_spells) => todo!("yet to be implemented"),
    }
}
