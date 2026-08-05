use crate::{
    compiler::weaves::Weave,
    values::{
        native_spells::{find_method, find_standalone_spell},
        spell::SpellInfo,
    },
    EiraVM, Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum NativeSpell {
    Standalone {
        name: String,
        info: SpellInfo,
    },
    Method {
        target: String,
        method: String,
        info: SpellInfo,
    },
}

impl NativeSpell {
    pub fn resolve(name: &str) -> Result<NativeSpell, String> {
        if let Some(def) = find_standalone_spell(name) {
            Ok(NativeSpell::Standalone {
                name: name.to_string(),
                info: (def.spell_info)(),
            })
        } else {
            Err(format!("Could'nt find a native spell for '{}'", name))
        }
    }

    pub fn resolve_methods(name: &str, weave: Weave) -> Result<NativeSpell, String> {
        let parts: Vec<&str> = name.split(':').collect();
        match parts.as_slice() {
            ["core", target, spell] => {
                if let Some(def) = find_method(target, spell) {
                    let info = (def.spell_info)(&weave)?;
                    Ok(NativeSpell::Method {
                        target: target.to_string(),
                        method: spell.to_string(),
                        info,
                    })
                } else {
                    Err(format!(
                        "The seal or spell '{}' is not defined for '{}' weave!",
                        spell, target
                    ))
                }
            }
            _ => Err(format!("Could'nt find a native spell for '{}'", name)),
        }
    }

    pub fn get_spell_info(spell: NativeSpell) -> Result<SpellInfo, String> {
        match spell {
            NativeSpell::Standalone { info, .. } => Ok(info),
            NativeSpell::Method { info, .. } => Ok(info),
        }
    }
}

pub fn dispatch(
    vm: &mut EiraVM,
    spell: NativeSpell,
    arg_start_idx: usize,
    argc: usize,
) -> Result<Value, String> {
    match spell {
        NativeSpell::Standalone { name, .. } => {
            let def = find_standalone_spell(&name)
                .ok_or_else(|| format!("Unknown standalone native spell '{}'", name))?;
            (def.handler)(vm, arg_start_idx, argc)
        }
        NativeSpell::Method { target, method, .. } => {
            let def = find_method(&target, &method)
                .ok_or_else(|| format!("Unknown method '{}.{}'", target, method))?;
            (def.handler)(vm, arg_start_idx, argc)
        }
    }
}
