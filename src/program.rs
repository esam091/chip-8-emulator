
#[path = "./instruction.rs"] mod instruction;
use std::fs;

use instruction::{Instruction, parse_opcode};

pub enum UIAction<'a> {
    ClearScreen,
    Draw(&'a [[bool; 64]; 32]),
}

pub struct Program {
    memory: [u8; 4096],
    program_counter: usize,

    registers: [u8; 16],
    i: u16,
    pixel_buffer: [[bool; 64]; 32],
}

impl Program {
    pub fn load(file_name: &String) -> Result<Program, String> {
        let bytes = fs::read(file_name).map_err(|_| format!("Read failed from {}", file_name))?;

        let mut memory = [0 as u8; 4096];

        for index in 0..bytes.len() {
            memory[512 + index] = bytes[index];
        }

        Ok(Program {
            memory,
            program_counter: 512,
            registers: [0; 16],
            i: 0,
            pixel_buffer: [[false; 64]; 32],
        })
    }

    pub fn step(&mut self) -> Option<UIAction> {
        let a = self.memory[self.program_counter];
        let b = self.memory[self.program_counter + 1];

        let instruction = ((a as u16) << 8) | b as u16;

        let opcode = parse_opcode(instruction);
        println!("instruction: {:#04x?}, opcode {:02x?}", instruction, opcode);

        if let Some(opcode) = opcode {
            match opcode {
                Instruction::ClearScreen => {
                    self.pixel_buffer = [[false; 64]; 32];
                    self.program_counter += 2;

                    return Some(UIAction::ClearScreen);
                }

                Instruction::StoreAddrToI(addr) => {
                    self.program_counter += 2;
                    self.i = addr;

                    return None;
                }

                Instruction::SetV { register, value } => {
                    self.registers[register as usize] = value;
                    self.program_counter += 2;
                }

                Instruction::Draw {
                    register_x,
                    register_y,
                    bytes,
                } => {
                    let x = self.registers[register_x as usize] % 64;
                    let mut y = self.registers[register_y as usize] % 32;

                    // println!("start drawing at {}, {}", x, y);

                    // try using self.vf to simplify the code
                    self.registers[0xf] = 0;

                    for index in 0..bytes {
                        if y >= 32 {
                            break;
                        }

                        let location = self.i as usize + index as usize;
                        let sprite_bytes = self.memory[location];

                        println!(
                            "extracting sprite at {:02x?}, value: {:#08b}",
                            location, sprite_bytes
                        );

                        let mut current_x = x;
                        for col in 0..8 {
                            if current_x >= 64 {
                                break;
                            }

                            let address = &mut self.pixel_buffer[y as usize][current_x as usize];
                            let pixel_value = *address as u8;

                            let sprite_value = sprite_bytes & (1 << (7 - col));

                            if sprite_value != 0 {
                                if pixel_value == 1 {
                                    *address = false;
                                    self.registers[0xf] = 1;
                                } else {
                                    println!("turn on at {}, {}", current_x, y);
                                    *address = true;
                                }
                            }

                            current_x += 1;
                        }

                        y += 1
                    }

                    self.program_counter += 2;

                    return Some(UIAction::Draw(&self.pixel_buffer));
                }

                Instruction::AddToRegister { register, value } => {
                    self.registers[register as usize] += value;
                    self.program_counter += 2;
                }

                Instruction::JumpToAddress(address) => {
                    self.program_counter = address as usize;
                }

                _ => return None,
            }
        }

        return None;
    }
}