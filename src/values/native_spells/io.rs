use std::rc::Rc;

use crate::Value;

pub fn read_line() -> Result<Value, String> {
    let buf = &mut String::new();
    match std::io::stdin().read_line(buf) {
        Ok(_) => Ok(Value::String(Rc::new(buf.trim().to_owned()))),
        Err(_) => Err("OS said no.".to_owned()),
    }
}
