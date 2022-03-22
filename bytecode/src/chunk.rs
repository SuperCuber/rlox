use crate::value::Value;
use encode_instruction::EncodeInstruction;

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

    pub fn write_code(&mut self, instruction: OpCode, line: usize) {
        instruction.encode(&mut self.code);
        if let Some((last_line, _)) = self.lines.last() {
            if *last_line < line {
                self.lines.push((line, self.code.len() - 1));
            }
        } else {
            // first instruction
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
        if let Some((ans, len)) = OpCode::decode(&self.code[offset..]) {
            match ans {
                OpCode::Constant(constant) => {
                    println!(
                        "{:-16} {} '{:?}'",
                        "CONSTANT", constant, self.constants[constant as usize]
                    );
                }
                OpCode::LargeConstant(constant) => {
                    println!(
                        "{:-16} {} '{:?}'",
                        "L_CONSTANT", constant, self.constants[constant as usize]
                    );
                }
                OpCode::Add => {
                    println!("ADD");
                }
                OpCode::Subtract => {
                    println!("SUBTRACT");
                }
                OpCode::Multiply => {
                    println!("MULTIPLY");
                }
                OpCode::Divide => {
                    println!("DIVIDE");
                }
                OpCode::Negate => {
                    println!("NEGATE");
                }
                OpCode::Return => {
                    println!("RETURN");
                }
            }
            len
        } else {
            println!("Unknown opcode {}", self.code[offset]);
            1
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

#[derive(Debug, Clone, Copy, encode_instruction_derive::EncodeInstruction)]
pub enum OpCode {
    Constant(u8),
    LargeConstant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}
