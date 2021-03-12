pub mod instruction;
pub mod program;

use std::env;
use std::{convert::TryInto, time::Duration};

use program::{Machine, UIAction, NUM_COLS, NUM_ROWS};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

const SCALE: u32 = 10;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let file_name = &args[1];

    let mut machine = Machine::load(file_name)?;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "CHIP-8 Emulator",
            (NUM_COLS as u32) * SCALE,
            (NUM_ROWS as u32) * SCALE,
        )
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

        if let Some(ui_action) = machine.step() {
            match ui_action {
                UIAction::ClearScreen => {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas.clear();
                }

                UIAction::Draw(pixel_buffer) => {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas.clear();

                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    for y in 0..NUM_ROWS {
                        for x in 0..NUM_COLS {
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
        ::std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
