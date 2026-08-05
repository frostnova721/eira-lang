use std::rc::Rc;

use crate::{
    Value,
    compiler::{reagents::WovenReagent, weaves::Weave},
    values::{native_spells::NativeMethodDef, spell::SpellInfo},
};

pub const TEXT_METHODS: &[NativeMethodDef] = &[
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
            let Value::String(text) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(text.chars().count() as f64))
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
            let Value::String(text) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Bool(text.is_empty()))
        },
    },
    NativeMethodDef {
        name: "extract",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "extract".to_string(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(Weave::Num),
                ],
                release_weave: Weave::Text,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let idx_val = &vm.stack[arg_start_idx + 1];
            let Value::String(text) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            let Value::Number(idx) = idx_val else {
                return Err(format!(
                    "Expected Num as index for extract, got '{:?}'",
                    idx_val
                ));
            };
            if *idx < 0.0 {
                return Ok(Value::Emptiness);
            }
            let val = text
                .chars()
                .nth(*idx as usize)
                .map_or(Value::Emptiness, |c| {
                    Value::String(Rc::new(String::from(c)))
                });
            Ok(val)
        },
    },
    NativeMethodDef {
        name: "infuse",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "infuse".to_string(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(Weave::Text),
                ],
                release_weave: Weave::Text,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let other_val = &vm.stack[arg_start_idx + 1];
            let Value::String(s1) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            let Value::String(s2) = other_val else {
                return Err(format!(
                    "Expected Text as value for infuse, got '{:?}'",
                    other_val
                ));
            };
            Ok(Value::String(Rc::new(format!("{}{}", s1, s2))))
        },
    },
    NativeMethodDef {
        name: "to_upper",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "to_upper".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Text,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::String(text) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::String(Rc::new(text.to_uppercase())))
        },
    },
    NativeMethodDef {
        name: "to_lower",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "to_lower".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Text,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::String(text) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::String(Rc::new(text.to_lowercase())))
        },
    },
    NativeMethodDef {
        name: "trim",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "trim".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Text,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::String(text) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::String(Rc::new(text.trim().to_string())))
        },
    },
    NativeMethodDef {
        name: "contains",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "contains".to_string(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(Weave::Text),
                ],
                release_weave: Weave::Truth,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let sub_val = &vm.stack[arg_start_idx + 1];
            let Value::String(s1) = ego else {
                return Err(format!("Expected Text as ego receiver, got '{:?}'", ego));
            };
            let Value::String(s2) = sub_val else {
                return Err(format!(
                    "Expected Text as argument for contains, got '{:?}'",
                    sub_val
                ));
            };
            Ok(Value::Bool(s1.contains(s2.as_str())))
        },
    },
    NativeMethodDef {
        name: "clear",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "clear".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Text,
                upvalues: vec![],
            })
        },
        handler: |_vm, _arg_start_idx, _argc| {
            Ok(Value::String(Rc::new(String::new())))
        },
    },
];
