pub mod instruction;
pub mod program;

use std::{convert::TryInto, time::Duration};
use std::env;


use program::{Program, UIAction};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

// use instruction;


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

