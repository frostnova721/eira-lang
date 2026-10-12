use crate::compiler::{reagents::WovenReagent, weaves::Weave};
use crate::values::spell::SpellInfo;
use crate::{EiraVM, Value};

pub fn parse_num_info() -> SpellInfo {
    SpellInfo {
        name: "parseNum".to_string(),
        reagents: vec![WovenReagent { weave: Weave::Text }],
        release_weave: Weave::Num,
        upvalues: vec![],
    }
}

pub fn parse_num_handler(vm: &mut EiraVM, arg_start_idx: usize, _argc: usize) -> Result<Value, String> {
    let arg_val = &vm.stack[arg_start_idx];
    let text = arg_val
        .extract_string()
        .ok_or_else(|| format!("Expected a Text argument for parseNum, got '{:?}'", arg_val))?;
    text.trim()
        .parse::<f64>()
        .map(Value::Number)
        .map_err(|_| format!("parseNum could not convert {:?} to Num", text))
}

#[inline(always)]
pub fn floor(value: f64) -> f64 {
    value.floor()
}

#[inline(always)]
pub fn ceil(value: f64) -> f64 {
    value.ceil()
}

pub fn floor_info() -> SpellInfo {
    SpellInfo {
        name: "floor".to_string(),
        reagents: vec![WovenReagent {
            weave: Weave::Num,
        }],
        release_weave: Weave::Num,
        upvalues: vec![],
    }
}

pub fn floor_handler(vm: &mut EiraVM, arg_start_idx: usize, _argc: usize) -> Result<Value, String> {
    let arg_val = &vm.stack[arg_start_idx];
    let num = arg_val
        .extract_number()
        .ok_or_else(|| format!("Expected a Num argument for floor, got '{:?}'", arg_val))?;
    Ok(Value::Number(floor(num)))
}

pub fn ceil_info() -> SpellInfo {
    SpellInfo {
        name: "ceil".to_string(),
        reagents: vec![WovenReagent {
            weave: Weave::Num,
        }],
        release_weave: Weave::Num,
        upvalues: vec![],
    }
}

pub fn ceil_handler(vm: &mut EiraVM, arg_start_idx: usize, _argc: usize) -> Result<Value, String> {
    let arg_val = &vm.stack[arg_start_idx];
    let num = arg_val
        .extract_number()
        .ok_or_else(|| format!("Expected a Num argument for ceil, got '{:?}'", arg_val))?;
    Ok(Value::Number(ceil(num)))
}
