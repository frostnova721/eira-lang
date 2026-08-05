use crate::{
    compiler::{reagents::WovenReagent, weaves::Weave},
    values::{native_spells::NativeMethodDef, spell::SpellInfo},
    Value,
};

pub const NUM_METHODS: &[NativeMethodDef] = &[
    NativeMethodDef {
        name: "floor",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "floor".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Number(num) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(num.floor()))
        },
    },
    NativeMethodDef {
        name: "ceil",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "ceil".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Number(num) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(num.ceil()))
        },
    },
    NativeMethodDef {
        name: "abs",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "abs".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Number(num) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(num.abs()))
        },
    },
    NativeMethodDef {
        name: "round",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "round".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Number(num) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(num.round()))
        },
    },
    NativeMethodDef {
        name: "sqrt",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "sqrt".to_string(),
                reagents: vec![WovenReagent::new(weave.clone())],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let Value::Number(num) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            Ok(Value::Number(num.sqrt()))
        },
    },
    NativeMethodDef {
        name: "min",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "min".to_string(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(Weave::Num),
                ],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let other = &vm.stack[arg_start_idx + 1];
            let Value::Number(n1) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            let Value::Number(n2) = other else {
                return Err(format!("Expected Num as argument for min, got '{:?}'", other));
            };
            Ok(Value::Number(n1.min(*n2)))
        },
    },
    NativeMethodDef {
        name: "max",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "max".to_string(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(Weave::Num),
                ],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let other = &vm.stack[arg_start_idx + 1];
            let Value::Number(n1) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            let Value::Number(n2) = other else {
                return Err(format!("Expected Num as argument for max, got '{:?}'", other));
            };
            Ok(Value::Number(n1.max(*n2)))
        },
    },
    NativeMethodDef {
        name: "clamp",
        spell_info: |weave| {
            Ok(SpellInfo {
                name: "clamp".to_string(),
                reagents: vec![
                    WovenReagent::new(weave.clone()),
                    WovenReagent::new(Weave::Num),
                    WovenReagent::new(Weave::Num),
                ],
                release_weave: Weave::Num,
                upvalues: vec![],
            })
        },
        handler: |vm, arg_start_idx, _argc| {
            let ego = &vm.stack[arg_start_idx];
            let min_val = &vm.stack[arg_start_idx + 1];
            let max_val = &vm.stack[arg_start_idx + 2];
            let Value::Number(num) = ego else {
                return Err(format!("Expected Num as ego receiver, got '{:?}'", ego));
            };
            let Value::Number(min) = min_val else {
                return Err(format!("Expected Num as min for clamp, got '{:?}'", min_val));
            };
            let Value::Number(max) = max_val else {
                return Err(format!("Expected Num as max for clamp, got '{:?}'", max_val));
            };
            Ok(Value::Number(num.clamp(*min, *max)))
        },
    },
];
