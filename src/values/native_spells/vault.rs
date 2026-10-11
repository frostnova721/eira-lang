use crate::{
    Value,
    compiler::{reagents::WovenReagent, weaves::Weave},
    values::{deck::DeckObject, native_spells::NativeMethodDef, spell::SpellInfo},
};
use std::rc::Rc;
pub const VAULT_METHODS: &[NativeMethodDef] = &[
    NativeMethodDef {
        name: "size",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "size".into(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 1 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            Ok(Value::Number(vault.len() as f64))
        },
    },
    NativeMethodDef {
        name: "contains",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "contains".into(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(*key.clone()),
                ],
                release_weave: Weave::Truth,
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 2 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            Ok(Value::Bool(vault.contains(&vm.stack[start + 1])?))
        },
    },
    NativeMethodDef {
        name: "get",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "get".into(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(*key.clone()),
                ],
                release_weave: Weave::Maybe(value.clone()),
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 2 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            vault.get(&vm.stack[start + 1])
        },
    },
    NativeMethodDef {
        name: "remove",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "remove".into(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(*key.clone()),
                ],
                release_weave: Weave::Maybe(value.clone()),
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 2 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            vault.remove(&vm.stack[start + 1])
        },
    },
    NativeMethodDef {
        name: "insert",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "insert".into(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(*key.clone()),
                    WovenReagent::new(*value.clone()),
                ],
                release_weave: Weave::Empty,
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 3 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            vault.insert(&vm.stack[start + 1], vm.stack[start + 2].clone())?;
            Ok(Value::Emptiness)
        },
    },
    NativeMethodDef {
        name: "clear",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "clear".into(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Empty,
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 1 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            vault.clear();
            Ok(Value::Emptiness)
        },
    },
    NativeMethodDef {
        name: "keys",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "keys".into(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Deck(key.clone(), None),
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 1 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            Ok(Value::Deck(Rc::new(DeckObject::new(vault.keys(), None))))
        },
    },
    NativeMethodDef {
        name: "values",
        spell_info: |weave| {
            let Weave::Vault(key, value, _) = weave else {
                return Err("Expected Vault receiver.".into());
            };
            let _ = (key, value);
            Ok(SpellInfo {
                name: "values".into(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Deck(value.clone(), None),
                upvalues: vec![],
            })
        },
        handler: |vm, start, argc| {
            if argc != 1 {
                return Err("Wrong Vault method argument count.".into());
            }
            let Value::Vault(vault) = &vm.stack[start] else {
                return Err("Expected Vault receiver.".into());
            };
            Ok(Value::Deck(Rc::new(DeckObject::new(vault.values(), None))))
        },
    },
];
