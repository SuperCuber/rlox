use crate::value::Value;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    /// Line number, start instruction
    lines: Vec<(usize, usize)>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_code(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        if let Some((last_line, _)) = self.lines.last() {
            if *last_line < line {
                self.lines.push((line, self.code.len() - 1));
            }
        } else {
            // first instruction
            assert!(self.code.len() == 1);
            self.lines.push((line, 0))
        }
    }

    /// Returns the index of the new constant
    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    #[allow(dead_code)]
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        let mut offset = 0;
        while offset < self.code.len() {
            offset += self.disassemble_instruction(offset);
        }
    }

    /// Returns the length of the just-printed instruction
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");
        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", self.get_line(offset));
        }
        match OpCode::from_u8(self.code[offset]) {
            Some(OpCode::Constant) => {
                let constant = self.code[offset + 1];
                println!(
                    "{:-16} {} '{:?}'",
                    "CONSTANT", constant, self.constants[constant as usize]
                );
                2
            }
            Some(OpCode::Add) => {
                println!("ADD");
                1
            }
            Some(OpCode::Subtract) => {
                println!("SUBTRACT");
                1
            }
            Some(OpCode::Multiply) => {
                println!("MULTIPLY");
                1
            }
            Some(OpCode::Divide) => {
                println!("DIVIDE");
                1
            }
            Some(OpCode::Negate) => {
                println!("NEGATE");
                1
            }
            Some(OpCode::Return) => {
                println!("RETURN");
                1
            }
            None => {
                println!("Unknown opcode {}", self.code[offset]);
                1
            }
        }
    }

    #[allow(clippy::comparison_chain)]
    pub fn get_line(&self, offset: usize) -> usize {
        let mut last_line = 0;
        for (line, current_offset) in &self.lines {
            if offset == *current_offset {
                return *line;
            } else if offset < *current_offset {
                // we passed the correct offset
                return last_line;
            }
            last_line = *line;
        }
        last_line
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl OpCode {
    pub fn as_u8(self) -> u8 {
        match self {
            OpCode::Constant => 0,
            OpCode::Add => 1,
            OpCode::Subtract => 2,
            OpCode::Multiply => 3,
            OpCode::Divide => 4,
            OpCode::Negate => 5,
            OpCode::Return => 6,
            // Make sure from_u8 is synced
        }
    }

    pub fn from_u8(code: u8) -> Option<OpCode> {
        Some(match code {
            0 => OpCode::Constant,
            1 => OpCode::Add,
            2 => OpCode::Subtract,
            3 => OpCode::Multiply,
            4 => OpCode::Divide,
            5 => OpCode::Negate,
            6 => OpCode::Return,
            _ => return None,
        })
    }
}
