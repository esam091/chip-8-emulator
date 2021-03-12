use std::{convert::TryInto, fs, time::Duration};
use std::{env, usize};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Instruction {
    // 00E0
    ClearScreen,

    // ANNN
    StoreAddrToI(u16),

    // 6XNN
    SetV {
        register: u8,
        value: u8,
    },

    // DXYN
    Draw {
        register_x: u8,
        register_y: u8,
        bytes: u8,
    },

    // 7XNN
    AddToRegister {
        register: u8,
        value: u8,
    },

    // 1NNN
    JumpToAddress(u16),

    // 3XNN
    SkipIfEqual {
        register: u8,
        value: u8,
    },

    // 0NNN
    ExecuteSubroutine(u16),

    // 8XY0
    StoreYToX {
        register_x: u8,
        register_y: u8,
    },
}

fn split_opcode(instruction: u16) -> (u8, u8, u8, u8) {
    let code1 = instruction & 0xf;
    let code2 = (instruction & 0x00f0) >> 4;
    let code3 = (instruction & 0x0f00) >> 8;
    let code4 = (instruction & 0xf000) >> 12;

    return (
        code4.try_into().unwrap(),
        code3.try_into().unwrap(),
        code2.try_into().unwrap(),
        code1.try_into().unwrap(),
    );
}

fn combine_nibble2(a: u8, b: u8) -> u8 {
    (a << 4) ^ b
}

fn combine_nibble3(a: u8, b: u8, c: u8) -> u16 {
    ((a as u16) << 8) ^ ((b as u16) << 4) ^ c as u16
}

fn parse_opcode(instruction: u16) -> Option<Instruction> {
    match split_opcode(instruction) {
        (0x0, 0x0, 0xe, 0x0) => Some(Instruction::ClearScreen),
        (0x0, a, b, c) => Some(Instruction::ExecuteSubroutine(combine_nibble3(a, b, c))),
        (0xa, a, b, c) => Some(Instruction::StoreAddrToI(combine_nibble3(a, b, c))),
        (0x6, register, a, b) => Some(Instruction::SetV {
            register,
            value: combine_nibble2(a, b),
        }),
        (0xd, register_x, register_y, bytes) => Some(Instruction::Draw {
            register_x,
            register_y,
            bytes,
        }),
        (0x7, register, a, b) => Some(Instruction::AddToRegister {
            register,
            value: combine_nibble2(a, b),
        }),
        (0x1, a, b, c) => Some(Instruction::JumpToAddress(combine_nibble3(a, b, c))),
        (0x3, register, a, b) => Some(Instruction::SkipIfEqual {
            register,
            value: combine_nibble2(a, b),
        }),
        (0x8, register_x, register_y, 0) => Some(Instruction::StoreYToX {
            register_x,
            register_y,
        }),
        _ => None,
    }
}
enum UIAction<'a> {
    ClearScreen,
    Draw(&'a [[bool; 64]; 32]),
}

struct Program {
    memory: [u8; 4096],
    program_counter: usize,

    registers: [u8; 16],
    i: u16,
    pixel_buffer: [[bool; 64]; 32],
}

impl Program {
    fn load(file_name: &String) -> Result<Program, String> {
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

    fn step(&mut self) -> Option<UIAction> {
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

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let file_name = &args[1];

    let mut program = Program::load(file_name)?;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8 Emulator", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // let aa = &mut program;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                // Event::KeyDown {
                //     keycode: Some(Keycode::Right),
                //     ..
                // } => x += 10,
                _ => {}
            }
        }

        if let Some(ui_action) = program.step() {
            match ui_action {
                UIAction::ClearScreen => {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas.clear();
                }

                UIAction::Draw(pixel_buffer) => {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas.clear();

                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    for y in 0..32 {
                        for x in 0..64 {
                            if pixel_buffer[y][x] {
                                let x = (x * 10).try_into().map_err(|value| {
                                    format!("Failed converting {} to i32", value)
                                })?;
                                let y = (y * 10).try_into().map_err(|value| {
                                    format!("Failed converting {} to i32", value)
                                })?;

                                canvas.fill_rect(Rect::new(x, y, 10, 10)).unwrap();
                            }
                        }
                    }

                    canvas.present();
                }
            }
        }

        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        ::std::thread::sleep(Duration::new(1, 0));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_test() {
        assert_eq!(split_opcode(0xabcd), (0xa, 0xb, 0xc, 0xd));
        assert_eq!(split_opcode(0x839a), (0x8, 0x3, 0x9, 0xa));
    }

    #[test]
    fn opcode_test() {
        let instructions_and_opcodes: Vec<(u16, Instruction)> = vec![
            (0x00e0, Instruction::ClearScreen),
            (0xa22a, Instruction::StoreAddrToI(0x22a)),
            (
                0x600c,
                Instruction::SetV {
                    register: 0,
                    value: 0x0c,
                },
            ),
            (
                0xd01f,
                Instruction::Draw {
                    register_x: 0,
                    register_y: 1,
                    bytes: 0xf,
                },
            ),
            (
                0x7009,
                Instruction::AddToRegister {
                    register: 0,
                    value: 0x09,
                },
            ),
            (0x1228, Instruction::JumpToAddress(0x228)),
            (
                0x3c00,
                Instruction::SkipIfEqual {
                    register: 0xc,
                    value: 0x00,
                },
            ),
            (0x0038, Instruction::ExecuteSubroutine(0x038)),
            (
                0x8320,
                Instruction::StoreYToX {
                    register_x: 3,
                    register_y: 2,
                },
            ),
        ];

        for (instruction, opcode) in instructions_and_opcodes {
            assert_eq!(
                parse_opcode(instruction),
                Some(opcode),
                "Expecting instruction {:#04x?} to translate to opcode {:#04x?}",
                instruction,
                opcode
            );
        }
    }
}
