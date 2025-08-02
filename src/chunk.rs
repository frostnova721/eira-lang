use crate::value::Value;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
    current_register: u8,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: vec![],
            constants: vec![],
            lines: vec![],
            current_register: 0,
        }
    }

    /// Get next register which is available for use
    pub fn get_next_reg(&mut self) -> u8 {
        if self.current_register == u8::MAX {
            panic!("Maximum registers allocated! Register overflow?!")
        }

        if self.current_register != 0 {
            self.current_register += 1;
        }
        self.current_register
    }

    pub fn get_last_allocated_register(&self) -> u8 {
        self.current_register
    }

    /// Write a byte to the chunk
    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Add a constant to the constant pool
    pub fn add_constant(&mut self, value: Value) -> u16 {
        if self.constants.len() >= u16::MAX as usize {
            panic!("Thats a lot of constants! Max amount of constants has been reached.")
        }
        self.constants.push(value);
        self.constants
            .len()
            .try_into()
            .expect("Index went out of 16bit limit!")
    }
}
