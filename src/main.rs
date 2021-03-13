pub mod instruction;
pub mod program;

use std::env;
use std::{convert::TryInto, time::Duration};

use program::{Machine, PixelBuffer, NUM_COLS, NUM_ROWS};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::WindowCanvas};

use common_macros::hash_map;

const SCALE: u32 = 10;

fn draw_pixel_buffer(canvas: &mut WindowCanvas, pixel_buffer: &PixelBuffer) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for y in 0..NUM_ROWS {
        for x in 0..NUM_COLS {
            if pixel_buffer[y][x] {
                let x = (x * 10)
                    .try_into()
                    .map_err(|value| format!("Failed converting {} to i32", value))?;
                let y = (y * 10)
                    .try_into()
                    .map_err(|value| format!("Failed converting {} to i32", value))?;

                canvas.fill_rect(Rect::new(x, y, 10, 10)).unwrap();
            }
        }
    }

    canvas.present();

    Ok(())
}

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

    let keyboard_mappings = hash_map! {
        Keycode::Right => 0x6u8,
        Keycode::Down => 0x8,
        Keycode::Left => 0x4,
        Keycode::Up => 0x2,

        Keycode::Num1 => 0x1,
        Keycode::Num2 => 0x2,
        Keycode::Num3 => 0x3,
        Keycode::Q => 0x4,
        Keycode::W => 0x5,
        Keycode::E => 0x6,
        Keycode::A => 0x7,
        Keycode::S => 0x8,
        Keycode::D => 0x9,
        Keycode::X => 0x0,

        Keycode::Z => 0xa,
        Keycode::C => 0xb,
        Keycode::Num4 => 0xc,
        Keycode::R => 0xd,
        Keycode::F => 0xe,
        Keycode::V => 0xf,
    };

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                // TODO: handle key from 0 to F
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = keyboard_mappings.get(&keycode) {
                        machine.key_press(*key);
                    }
                }

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = keyboard_mappings.get(&keycode) {
                        machine.key_release(*key);
                    }
                }

                _ => {}
            }
        }

        machine.step();

        draw_pixel_buffer(&mut canvas, machine.get_pixel_buffer())?;

        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        ::std::thread::sleep(Duration::from_millis(17));
    }

    Ok(())
}
