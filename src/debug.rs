use crate::{runtime::{instruction::Instruction, operation::OpCode}, value::Value};


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
        let mut len = op.inst_len(); // or OpCode::inst_len(&op)

        // Special handling for Call instruction (variable length)
        if matches!(op, OpCode::Call) {
            // Call format: opcode(1) + dest(1) + callee(1) + arg_count(1) + args(arg_count)
            if i + 4 <= code.len() {
                let arg_count = code[i + 3];
                len = 4 + arg_count as usize;
            } else {
                println!(
                    "{:04}: Incomplete Call instruction at end of bytecode",
                    i
                );
                break;
            }
        }

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
