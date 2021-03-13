#[path = "./instruction.rs"]
mod instruction;
use std::{cmp::max, fs};

use instruction::{parse_opcode, Instruction};

pub const NUM_ROWS: usize = 32;
pub const NUM_COLS: usize = 64;

const MEMORY_SIZE: usize = 4096;
const PROGRAM_STARTING_ADDRESS: u16 = 512;
const FONT_STARTING_ADDRESS: usize = 0x50;
const FONT_BYTES: u32 = 5;

fn copy_font_data(memory: &mut [u8; MEMORY_SIZE]) {
    let font_data: Vec<u8> = vec![
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];

    for index in 0..font_data.len() {
        memory[FONT_STARTING_ADDRESS + index] = font_data[index];
    }
}

pub enum UIAction<'a> {
    ClearScreen,
    Draw(&'a [[bool; 64]; 32]),
}

pub type PixelBuffer = [[bool; NUM_COLS as usize]; NUM_ROWS as usize];

pub struct Machine {
    memory: [u8; MEMORY_SIZE],
    program_counter: u16,

    registers: [u8; 16],
    delay_timer: u8,
    i: u16,
    pixel_buffer: PixelBuffer,
    stack: Vec<u16>,
    key_is_pressed: [bool; 16],
    
    current_pressed_key: Option<u8>,
}

impl Machine {
    pub fn load(file_name: &String) -> Result<Machine, String> {
        let bytes = fs::read(file_name).map_err(|_| format!("Read failed from {}", file_name))?;

        let mut memory = [0 as u8; MEMORY_SIZE];

        for index in 0..bytes.len() {
            memory[PROGRAM_STARTING_ADDRESS as usize + index] = bytes[index];
        }

        copy_font_data(&mut memory);

        Ok(Machine {
            memory,
            program_counter: PROGRAM_STARTING_ADDRESS,
            registers: [0; 16],
            i: 0,
            pixel_buffer: [[false; NUM_COLS]; NUM_ROWS],
            stack: Vec::new(),
            key_is_pressed: [false; 16],
            current_pressed_key: None,
            delay_timer: 0,
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
                self.registers[register as usize] =
                    self.registers[register as usize].wrapping_add(value);

                return None;
            }

            Instruction::JumpToAddress(address) => {
                self.program_counter = address;
                return None;
            }

            Instruction::SkipIfNotEqual { register, value } => {
                if self.registers[register as usize] != value {
                    self.program_counter += 2;
                }

                return None;
            }

            Instruction::AddRegisterToI(register) => {
                self.i += self.registers[register as usize] as u16;
                return None;
            }

            Instruction::CallSubroutineAtAddress(address) => {
                self.stack.push(self.program_counter);
                self.program_counter = address;
                return None;
            }

            Instruction::ReturnFromSubroutine => {
                let return_address = self
                    .stack
                    .pop()
                    .expect("Returning from subroutine, but the stack is empty");
                self.program_counter = return_address;
                return None;
            }
            Instruction::SkipIfEqual { register, value } => {
                if self.registers[register as usize] == value {
                    self.program_counter += 2;
                }

                return None;
            }
            Instruction::StoreYToX {
                register_x,
                register_y,
            } => {
                self.registers[register_x as usize] = self.registers[register_y as usize];
                return None;
            }

            Instruction::SetRandomNumber { register, mask } => {
                let value = fastrand::u8(..) & mask;
                self.registers[register as usize] = value;

                return None;
            }
            Instruction::SkipIfRegistersEqual {
                register_x,
                register_y,
            } => {
                if self.registers[register_x as usize] == self.registers[register_y as usize] {
                    self.program_counter += 2;
                }

                return None;
            }
            Instruction::OrRegisters {
                register_x,
                register_y,
            } => {
                self.registers[register_x as usize] |= self.registers[register_y as usize];

                return None;
            }
            Instruction::AndRegisters {
                register_x,
                register_y,
            } => {
                self.registers[register_x as usize] &= self.registers[register_y as usize];

                return None;
            }
            Instruction::XorRegisters {
                register_x,
                register_y,
            } => {
                self.registers[register_x as usize] ^= self.registers[register_y as usize];

                return None;
            }
            Instruction::AddRegisters { register_x, register_y } => { 
                let value_x = self.registers[register_x as usize];
                let value_y = self.registers[register_y as usize];

                self.registers[0xf] = if value_x.checked_add(value_y) == None { 1 } else { 0 };
                self.registers[register_x as usize] = value_x.wrapping_add(value_y);
                
                return None;
            }
            Instruction::SubtractXMinusY { register_x, register_y } => {
                let value_x = self.registers[register_x as usize];
                let value_y = self.registers[register_y as usize];

                self.registers[0xf] = if value_x >= value_y { 1 } else { 0 };
                self.registers[register_x as usize] = value_x.wrapping_sub(value_y);

                return None;
            }
            Instruction::SubtractYMinusX { register_x, register_y } => {
                let value_x = self.registers[register_x as usize];
                let value_y = self.registers[register_y as usize];

                self.registers[0xf] = if value_y >= value_x { 1 } else { 0 };
                self.registers[register_x as usize] = value_y.wrapping_sub(value_x);

                return None;
            }
            Instruction::SkipIfRegistersNotEqual { register_x, register_y } => {
                if self.registers[register_x as usize] != self.registers[register_y as usize] {
                    self.program_counter += 2;
                }

                return None;
            }
            Instruction::ShiftRegisterLeft { register_x, register_y } => {
                // keypad test requires the CHIP48 behavior
                let value_y = self.registers[register_x as usize];

                self.registers[0xf] = value_y & (1 << 7);
                self.registers[register_x as usize] <<= 1;

                return None;
            }
            Instruction::ShiftRegisterRight { register_x, register_y } => {
                // keypad test requires the CHIP48 behavior
                let value_y = self.registers[register_x as usize];

                self.registers[0xf] = value_y & 1;
                self.registers[register_x as usize]  >>= 1;

                return None;
            }
            Instruction::LoadRegisters(final_register) => {  
                for register in 0 ..= final_register {
                    self.registers[register as usize] = self.memory[self.i as usize + register as usize];
                }
                // self.i += final_register as u16 + 1;

                return None;
            }
            Instruction::SaveRegisters(final_register) => {
                for register in 0 ..= final_register {
                    self.memory[self.i as usize + register as usize] = self.registers[register as usize];
                }

                // self.i += final_register as u16 + 1;
                return None;
            }
            Instruction::SetIToFontLocation(register) => {
                let font_character = self.registers[register as usize] as u16;
                self.i = FONT_STARTING_ADDRESS as u16 + 5 * font_character;

                return None;
            }
            Instruction::HaltAndGetKey(register) => { 
                match self.current_pressed_key {
                    None => self.program_counter -= 2,
                    Some(key) => self.registers[register as usize] = key
                }

                return None;
            }
            Instruction::SetDelayTimerFromRegister(register) => { 
                self.delay_timer = self.registers[register as usize];
                return None;
            }
            Instruction::SetRegisterFromDelayTimer(register) => { 
                self.registers[register as usize] = self.delay_timer;
                return None;
            }
        }
    }

    pub fn step(&mut self) -> Option<UIAction> {
        let a = self.memory[self.program_counter as usize];
        let b = self.memory[self.program_counter as usize + 1];

        let opcode = ((a as u16) << 8) | b as u16;

        let instruction = parse_opcode(opcode);
        self.program_counter += 2;
        println!("instruction: {:#04x?}, opcode {:02x?}", opcode, instruction);

        self.delay_timer = self.delay_timer.checked_sub(1).unwrap_or(0);

        return instruction
            .map(move |instruction| self.handle_instruction(instruction))
            .expect(format!("Failed to translate opcode: {:#02x?}, either the opcode is not supported yet, or there is a bug in the interpreter", opcode).as_str());
    }

    pub fn key_press(&mut self, key: u8) {
        self.current_pressed_key = Some(key);
    }

    pub fn key_release(&mut self, key: u8) {
        self.current_pressed_key = None;
    }

    pub fn get_pixel_buffer(&self) -> &PixelBuffer {
        &self.pixel_buffer
    }    
}
