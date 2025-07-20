use crate::compiler;

pub fn start() {
    compiler::compile();
    loop {
        println!("Vm running");
        break;
    }
}