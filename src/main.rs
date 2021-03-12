use std::{convert::TryInto, fs, ops::BitXor};
use std::env;

enum OpCode {
  ClearScreen,

}

fn split_instruction(instruction: u16) -> (u8, u8, u8, u8) {
  let code1 = instruction & 0xf;
  let code2 = (instruction & 0x00f0) >> 4;
  let code3 = (instruction & 0x0f00) >> 8;
  let code4 = (instruction & 0xf000) >> 12;

  return (
    code4.try_into().unwrap(), 
    code3.try_into().unwrap(),
    code2.try_into().unwrap(), 
    code1.try_into().unwrap()
  );
}

fn instruction_to_opcode(instruction: u16) -> OpCode {
  match instruction {
    0x0e00 => OpCode::ClearScreen,
    _ => panic!("Unhandled instruction: {:#04x?}", instruction)
  }
}

fn instructions_to_opcodes(instructions: Vec<u16>) -> Vec<OpCode> {
  let mut opcodes = Vec::new();

  for instruction in instructions {
    opcodes.push(instruction_to_opcode(instruction));
  }

  opcodes
}

fn main() -> Result<(), String> {
  let args: Vec<String> = env::args().collect();

  let file_name = &args[1];

  // Todo
  let bytes = fs::read(file_name)
    .map_err(|err| format!("Read failed from {}", file_name))?;

  // TODO: handle odd number of bytes
  let u16_vec: Vec<u16> = bytes.chunks_exact(2)
    .map(|a| u16::from_be_bytes([a[0], a[1]]))
    .collect();

  println!("{:04x?}", u16_vec);

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn split_test() {
    assert_eq!(split_instruction(0xabcd), (0xa, 0xb, 0xc, 0xd));
    assert_eq!(split_instruction(0x839a), (0x8, 0x3, 0x9, 0xa));
  }
}