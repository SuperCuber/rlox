use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
};

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

macro_rules! binary_op {
    ($self:ident, $op:tt) => {{
        let b = $self.stack_pop();
        let a = $self.stack_pop();
        $self.stack_push(a $op b);
    }};
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: [Value::default(); STACK_MAX],
            stack_top: 0,
        }
    }

    pub fn interpret(&mut self) -> Result<(), VMError> {
        self.run()
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            #[cfg(feature = "tracing")]
            {
                print!("          ");
                for value in self.stack[0..self.stack_top].iter().rev() {
                    print!("[ {value:?} ]");
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            match self.read_instruction().expect("next instruction") {
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack_push(constant);
                }
                OpCode::Add => binary_op!(self, +),
                OpCode::Subtract => binary_op!(self, -),
                OpCode::Multiply => binary_op!(self, *),
                OpCode::Divide => binary_op!(self, /),
                OpCode::Negate => {
                    let v = self.stack_pop();
                    self.stack_push(-v);
                }
                OpCode::Return => {
                    println!("{:?}", self.stack_pop());
                    return Ok(());
                }
            }
        }
    }

    // Chunk util

    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte() as usize;
        self.chunk.constants[idx]
    }

    fn read_instruction(&mut self) -> Option<OpCode> {
        OpCode::from_u8(self.read_byte())
    }

    fn read_byte(&mut self) -> u8 {
        let res = self.chunk.code[self.ip];
        self.ip += 1;
        res
    }

    // Stack util

    fn stack_push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn stack_pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }
}

#[derive(Debug)]
pub enum VMError {
    Compile,
    Runtime,
}
