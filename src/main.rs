mod instruction;

use std::{convert::TryInto, fs, time::Duration};
use std::{env, usize};

use instruction::{Instruction, parse_opcode};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

// use instruction;

// pub use instruction;
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

