use std::io::{self, Write};
use std::rc::Rc;

use crate::compiler::{reagents::WovenReagent, weaves::Weave};
use crate::values::spell::SpellInfo;
use crate::{EiraVM, Value};

pub fn read_line(prompt: Option<&str>) -> Result<Value, String> {
    if let Some(p) = prompt {
        print!("{}", p);
        let _ = io::stdout().flush();
    }

    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(_) => Ok(Value::String(Rc::new(buf.trim().to_owned()))),
        Err(_) => Err("OS said no.".to_owned()),
    }
}

pub fn listen_info() -> SpellInfo {
    SpellInfo {
        name: "listen".to_string(),
        reagents: vec![],
        release_weave: Weave::Text,
        upvalues: vec![],
    }
}

pub fn listen_handler(_vm: &mut EiraVM, _arg_start_idx: usize, _argc: usize) -> Result<Value, String> {
    read_line(None)
}

pub fn ask_info() -> SpellInfo {
    SpellInfo {
        name: "ask".to_string(),
        reagents: vec![WovenReagent {
            weave: Weave::Text,
        }],
        release_weave: Weave::Text,
        upvalues: vec![],
    }
}

pub fn ask_handler(vm: &mut EiraVM, arg_start_idx: usize, _argc: usize) -> Result<Value, String> {
    let prompt_val = &vm.stack[arg_start_idx];
    let prompt_str = prompt_val
        .extract_string()
        .ok_or_else(|| format!("Expected a Text prompt, got '{:?}'", prompt_val))?;
    read_line(Some(&prompt_str))
}
