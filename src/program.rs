#[path = "./instruction.rs"]
mod instruction;
use std::fs;

use instruction::{parse_opcode, Instruction};

pub const NUM_ROWS: usize = 32;
pub const NUM_COLS: usize = 64;

const MEMORY_SIZE: usize = 4096;
const PROGRAM_STARTING_ADDRESS: usize = 512;

pub enum UIAction<'a> {
    ClearScreen,
    Draw(&'a [[bool; 64]; 32]),
}

pub struct Machine {
    memory: [u8; MEMORY_SIZE],
    program_counter: usize,

    registers: [u8; 16],
    i: u16,
    pixel_buffer: [[bool; NUM_COLS as usize]; NUM_ROWS as usize],
}

impl Machine {
    pub fn load(file_name: &String) -> Result<Machine, String> {
        let bytes = fs::read(file_name).map_err(|_| format!("Read failed from {}", file_name))?;

        let mut memory = [0 as u8; MEMORY_SIZE];

        for index in 0..bytes.len() {
            memory[PROGRAM_STARTING_ADDRESS + index] = bytes[index];
        }

        Ok(Machine {
            memory,
            program_counter: PROGRAM_STARTING_ADDRESS,
            registers: [0; 16],
            i: 0,
            pixel_buffer: [[false; NUM_COLS]; NUM_ROWS],
        })
    }

    fn handle_instruction(&mut self, instruction: Instruction) -> Option<UIAction> {
        match instruction {
            Instruction::ClearScreen => {
                self.pixel_buffer = [[false; 64]; 32];

                return Some(UIAction::ClearScreen);
            }

            Instruction::StoreAddrToI(addr) => {
                self.i = addr;

                return None;
            }

            Instruction::SetV { register, value } => {
                self.registers[register as usize] = value;

                return None;
            }

            Instruction::Draw {
                register_x,
                register_y,
                bytes,
            } => {
                let x = self.registers[register_x as usize] % (NUM_COLS as u8);
                let mut y = self.registers[register_y as usize] % (NUM_ROWS as u8);

                // println!("start drawing at {}, {}", x, y);

                // try using self.vf to simplify the code
                self.registers[0xf] = 0;

                for index in 0..bytes {
                    if y >= 32 {
                        break;
                    }

                    let location = self.i as usize + index as usize;
                    let sprite_bytes = self.memory[location];

                    // println!(
                    //     "extracting sprite at {:02x?}, value: {:#08b}",
                    //     location, sprite_bytes
                    // );

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
                                // println!("turn on at {}, {}", current_x, y);
                                *address = true;
                            }
                        }

                        current_x += 1;
                    }

                    y += 1
                }

                return Some(UIAction::Draw(&self.pixel_buffer));
            }

            Instruction::AddToRegister { register, value } => {
                self.registers[register as usize] += value;
                return None;
            }

            Instruction::JumpToAddress(address) => {
                self.program_counter = address as usize;
                return None;
            }

            _ => return None,
        }
    }

    pub fn step(&mut self) -> Option<UIAction> {
        let a = self.memory[self.program_counter];
        let b = self.memory[self.program_counter + 1];

        let opcode = ((a as u16) << 8) | b as u16;

        let instruction = parse_opcode(opcode);
        self.program_counter += 2;
        println!("instruction: {:#04x?}, opcode {:02x?}", opcode, instruction);

        return instruction
            .map(|instruction| self.handle_instruction(instruction))
            .expect(format!("Failed to translate opcode: {:#02x?}, either the opcode is not supported yet, or there is a bug in the interpreter", opcode).into());
    }
}
