use std::{convert::TryInto, fs, ops::Shl};
use std::env;

#[derive(Debug, PartialEq, Eq)]
enum OpCode {
  ClearScreen,
  StoreAddrToI(u16),
  SetV { register: u8, value: u16 },
  Draw { register_x: u8, register_y: u8, bytes: u8 },
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

fn combine2(a: u8, b: u8) -> u16 {
  (a as u16) << 4 ^ (b as u16)
}

fn combine3(a: u8, b: u8, c: u8) -> u16 {
  ((a as u16) << 8) ^ ((b as u16) << 4) ^ c as u16
}

fn instruction_to_opcode(instruction: u16) -> OpCode {
  match split_instruction(instruction) {
    (0x0, 0x0, 0xe, 0x0) => OpCode::ClearScreen,
    (0xa, a, b, c) => OpCode::StoreAddrToI(combine3(a, b, c)),
    (0x6, register, a, b) => OpCode::SetV { register, value: combine2(a, b) },
    (0xd, register_x, register_y, bytes) => OpCode::Draw { register_x, register_y, bytes },
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

  let opcodes = instructions_to_opcodes(u16_vec);

  println!("{:04x?}", opcodes);

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

  #[test]
  fn opcode_test() {
    let instructions_and_opcodes: Vec<(u16, OpCode)> = vec![
      (0x00e0, OpCode::ClearScreen),
      (0xa22a, OpCode::StoreAddrToI(0x22a)),
      (0x600c, OpCode::SetV { register: 0, value: 0x0c }),
      (0xd01f, OpCode::Draw { register_x: 0, register_y: 1, bytes: 0xf }),
    ];

    for (instruction, opcode) in instructions_and_opcodes {
      assert_eq!(instruction_to_opcode(instruction), opcode, "Expecting instruction {:#04x?} to translate to opcode {:#04x?}", instruction, opcode);
    }
  }
}