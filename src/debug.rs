use crate::{runtime::{Instruction, OpCode}, values::Value};


pub fn print_instructions(spell_name: &str,instructions: &Vec<Instruction>, constants: &Vec<Value>) {
    println!("Constant table: {:?}", constants);
    println!("Instructions - '{}': ({})\n\n==START==\n", spell_name, instructions.len());
    for inst in instructions {
        println!("{}", inst.to_string());
    }
    println!("\n==END==\n");
}

pub fn print_byte_code(code: &Vec<u8>) {
    println!("BYTE_CODE: ({})\n\n==START==\n", code.len());
    let mut i = 0;
    while i < code.len() {
        let byte = code[i];
        let op = OpCode::try_from(byte).unwrap();
        let len = op.inst_len(); // or OpCode::inst_len(&op)

        if i + len > code.len() {
            println!(
                "{:04}: Incomplete instruction at end of bytecode (expected {}, but only {} left)",
                i,
                len,
                code.len() - i
            );
            break;
        }

        print!("{:04}: ", i); // address padding
        for j in i..i + len {
            print!("{:02X} ", code[j]); // hex format looks cooler
        }
        println!("-> {:?}", op); // optional: human-readable op name

        i += len;
    }
    println!("\n==END==\n");
}
