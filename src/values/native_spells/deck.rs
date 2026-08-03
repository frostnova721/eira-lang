use crate::{
    EiraVM, Value,
    compiler::{reagents::WovenReagent, weaves::Weave},
    values::spell::SpellInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DeckSpells {
    Push(SpellInfo),
    Size(SpellInfo),
    MaxSize(SpellInfo),
    Pop(SpellInfo),
    Clear(SpellInfo),
}

impl DeckSpells {
    pub fn resolve(name: &str, deck_weave: Weave) -> Result<DeckSpells, String> {
        let inner_weave = match &deck_weave {
            Weave::Deck(inner, _) => inner.clone(),
            _ => {
                return Err(
                    "This was supposed to be called for resolving spells for deck!".to_string(),
                );
            }
        };

        match name {
            "push" => Ok(DeckSpells::Push(SpellInfo {
                name: "push".to_string(),
                reagents: vec![
                    WovenReagent::new(deck_weave),
                    WovenReagent::new(*inner_weave.clone()),
                ],
                release_weave: *inner_weave,
                upvalues: vec![],
            })),
            "size" => Ok(DeckSpells::Size(SpellInfo {
                name: "size".to_string(),
                reagents: vec![WovenReagent::new(deck_weave)],
                release_weave: Weave::Num,
                upvalues: vec![],
            })),
            "max_size" => Ok(DeckSpells::MaxSize(SpellInfo {
                name: "max_size".to_string(),
                reagents: vec![WovenReagent::new(deck_weave)],
                release_weave: Weave::Empty,
                upvalues: vec![],
            })),
            "pop" => Ok(DeckSpells::Pop(SpellInfo {
                name: "pop".to_string(),
                reagents: vec![WovenReagent::new(deck_weave)],
                release_weave: Weave::Maybe(inner_weave),
                upvalues: vec![],
            })),
            "clear" => Ok(DeckSpells::Clear(SpellInfo {
                name: "clear".to_string(),
                reagents: vec![WovenReagent::new(deck_weave)],
                release_weave: Weave::Empty,
                upvalues: vec![],
            })),
            _ => Err(format!("Spell '{}' not found.", name)),
        }
    }

    pub fn get_spell_info(spell: &DeckSpells) -> SpellInfo {
        match spell {
            DeckSpells::Push(si)
            | DeckSpells::Size(si)
            | DeckSpells::MaxSize(si)
            | DeckSpells::Pop(si)
            | DeckSpells::Clear(si) => si.clone(),
        }
    }

    pub fn dispatch(
        spell: &DeckSpells,
        vm: &mut EiraVM,
        arg_start_idx: usize,
        _argc: usize,
    ) -> Result<Value, String> {
        let ego = &vm.stack[arg_start_idx];

        let Value::Deck(deck) = ego else {
            return Err(format!(
                "Expected a Deck as the ego reciever, got '{:?}'",
                ego
            ));
        };

        match spell {
            DeckSpells::Push(_) => {
                let val = vm.stack[arg_start_idx + 1].clone();
                deck.items.borrow_mut().push(val);
                Ok(Value::Emptiness)
            }
            DeckSpells::Size(_) => Ok(Value::Number(deck.items.borrow().len() as f64)),
            DeckSpells::MaxSize(_) => Ok(Value::Number(deck.capacity.unwrap_or(0) as f64)),
            DeckSpells::Pop(_) => Ok(deck.items.borrow_mut().pop().unwrap_or(Value::Emptiness)),
            DeckSpells::Clear(_) => {
                deck.items.borrow_mut().clear();
                Ok(Value::Emptiness)
            }
        }
    }
}
