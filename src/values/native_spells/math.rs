use crate::compiler::{reagents::WovenReagent, weaves::Weave};
use crate::values::spell::SpellInfo;
use crate::{EiraVM, Value};

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
