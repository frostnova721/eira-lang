use crate::{
    compiler::{reagents::WovenReagent, weaves::Weave},
    values::{native_spells::NativeMethodDef, spell::SpellInfo},
    Value,
};

pub const DECK_METHODS: &[NativeMethodDef] = &[
    NativeMethodDef {
        name: "push",
        spell_info: |weave| {
            let inner = match weave {
                Weave::Deck(inner, _) => inner.clone(),
                _ => return Err("Expected Deck weave for 'push'".to_string()),
            };
            Ok(SpellInfo {
                name: "push".to_string(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(*inner),
                ],
                release_weave: Weave::Empty,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let val = vm.stack[arg_start_idx + 1].clone();
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            if let Some(cap) = deck.capacity {
                if deck.items.borrow().len() >= cap {
                    return Err(format!(
                        "Index out of bounds while adding element to a deck. Tried to push beyond deck capacity of {}.",
                        cap
                    ));
                }
            }
            deck.items.borrow_mut().push(val);
            Ok(Value::Emptiness)
        },
    },
    NativeMethodDef {
        name: "size",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "size".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(deck.items.borrow().len() as f64))
        },
    },
    NativeMethodDef {
        name: "max_size",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "max_size".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(deck.capacity.unwrap_or(0) as f64))
        },
    },
    NativeMethodDef {
        name: "pop",
        spell_info: |weave| {
            let inner = match weave {
                Weave::Deck(inner, _) => inner.clone(),
                _ => return Err("Expected Deck weave for 'pop'".to_string()),
            };
            Ok(SpellInfo {
                name: "pop".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Maybe(inner),
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            Ok(deck.items.borrow_mut().pop().unwrap_or(Value::Emptiness))
        },
    },
    NativeMethodDef {
        name: "clear",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "clear".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Empty,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            deck.items.borrow_mut().clear();
            Ok(Value::Emptiness)
        },
    },
    NativeMethodDef {
        name: "is_empty",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "is_empty".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Truth,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Bool(deck.items.borrow().is_empty()))
        },
    },
    NativeMethodDef {
        name: "first",
        spell_info: |weave| {
            let inner = match weave {
                Weave::Deck(inner, _) => inner.clone(),
                _ => return Err("Expected Deck weave for 'first'".to_string()),
            };
            Ok(SpellInfo {
                name: "first".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Maybe(inner),
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            Ok(deck.items.borrow().first().cloned().unwrap_or(Value::Emptiness))
        },
    },
    NativeMethodDef {
        name: "last",
        spell_info: |weave| {
            let inner = match weave {
                Weave::Deck(inner, _) => inner.clone(),
                _ => return Err("Expected Deck weave for 'last'".to_string()),
            };
            Ok(SpellInfo {
                name: "last".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Maybe(inner),
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            Ok(deck.items.borrow().last().cloned().unwrap_or(Value::Emptiness))
        },
    },
    NativeMethodDef {
        name: "reverse",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "reverse".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Empty,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Deck(deck) = ego else {
                return Err(format!("Expected a Deck as the ego receiver, got '{:?}'", ego));
            };
            deck.items.borrow_mut().reverse();
            Ok(Value::Emptiness)
        },
    },
];
