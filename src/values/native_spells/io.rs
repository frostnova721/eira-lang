use std::rc::Rc;
use std::io::{self, Write};

use crate::Value;

pub fn read_line(prompt: Option<&str>) -> Result<Value, String> {
    if let Some(p) = prompt {
        print!("{}", p);
        let _ = io::stdout().flush();
    }
    
    let buf = &mut String::new();
    match io::stdin().read_line(buf) {
        Ok(_) => Ok(Value::String(Rc::new(buf.trim().to_owned()))),
        Err(_) => Err("OS said no.".to_owned()),
    }
}
