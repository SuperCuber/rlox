use chunk::{Chunk, OpCode};
use vm::VM;

mod chunk;
mod value;
mod vm;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write_code(OpCode::Constant.as_u8(), 123);
    chunk.write_code(constant as u8, 123);

    let constant = chunk.add_constant(3.4);
    chunk.write_code(OpCode::Constant.as_u8(), 123);
    chunk.write_code(constant as u8, 123);

    chunk.write_code(OpCode::Add.as_u8(), 123);

    let constant = chunk.add_constant(5.6);
    chunk.write_code(OpCode::Constant.as_u8(), 123);
    chunk.write_code(constant as u8, 123);

    chunk.write_code(OpCode::Divide.as_u8(), 123);

    chunk.write_code(OpCode::Negate.as_u8(), 123);
    chunk.write_code(OpCode::Return.as_u8(), 124);
    // chunk.disassemble("potato");
    let mut vm = VM::new(chunk);
    vm.interpret().unwrap();
}
