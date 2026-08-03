use crate::{
    EiraVM, Value,
    compiler::{reagents::WovenReagent, weaves::Weave},
    values::spell::SpellInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub enum NumSpells {
    Floor(SpellInfo),
    Ceil(SpellInfo),
    Abs(SpellInfo),
    Round(SpellInfo),
    // Min(SpellInfo),
    // Max(SpellInfo),
    // Clamp(SpellInfo),
    // Lerp(SpellInfo),
}

impl NumSpells {
    pub fn resolve(name: &str) -> Result<NumSpells, String> {
        match name {
            "floor" => Ok(NumSpells::Floor(SpellInfo {
                name: "floor".to_string(),
                reagents: vec![WovenReagent::new(Weave::Num)],
                release_weave: Weave::Num,
                upvalues: vec![],
            })),
            "ceil" => Ok(NumSpells::Ceil(SpellInfo {
                name: "ceil".to_string(),
                reagents: vec![WovenReagent::new(Weave::Num)],
                release_weave: Weave::Num,
                upvalues: vec![],
            })),
            _ => Err(format!("Spell '{}' not found.", name)),
        }
    }

    pub fn get_spell_info(spell: &NumSpells) -> SpellInfo {
        match spell {
            NumSpells::Floor(si)
            | NumSpells::Ceil(si)
            | NumSpells::Abs(si)
            | NumSpells::Round(si) => si.clone(),
        }
    }

    pub fn dispatch(
        spell: &NumSpells,
        vm: &mut EiraVM,
        arg_start_idx: usize,
        _argc: usize,
    ) -> Result<Value, String> {
        let ego = &vm.stack[arg_start_idx];

        let Value::Number(num) = ego else {
            return Err(format!(
                "Expected a Number as the ego reciever, got '{:?}'",
                ego
            ));
        };

        match spell {
            NumSpells::Floor(_) => Ok(Value::Number(num.floor())),
            NumSpells::Ceil(_) => Ok(Value::Number(num.ceil())),
            NumSpells::Abs(_) => Ok(Value::Number(num.abs())),
            NumSpells::Round(_) => Ok(Value::Number(num.round())),
        }
    }
}
