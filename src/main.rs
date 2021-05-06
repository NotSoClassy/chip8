extern crate sdl2;
extern crate rand;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::rect::Rect;

use std::time::Duration;
use std::thread::sleep;
use std::io::Read;
use std::fs::File;
use std::env;

mod chip8;

use chip8::{ Chip8, CHIP8_HEIGHT, CHIP8_WIDTH };

const SCALE: u32 = 20;
const SCREEN_WIDTH: u32 = (CHIP8_WIDTH as u32) * SCALE;
const SCREEN_HEIGHT: u32 = (CHIP8_HEIGHT as u32) * SCALE;
const WAIT_TIME: Duration = Duration::from_millis(2);

const KEYS: [Keycode; 16] = [
    Keycode::X,
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::Z,
    Keycode::C,
    Keycode::Num4,
    Keycode::R,
    Keycode::F,
    Keycode::V
];

fn main() -> Result<(), String>{
    let name = env::args().nth(1).expect("expected file");
    let mut file = File::open(name).expect("could not open file");
    let mut buf = Vec::new();

    file.read_to_end(&mut buf).expect("Failure to read file");
    //let size = buf.len();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut event_pump = sdl_context.event_pump()?;
    let window = video_subsystem
        .window("Chip-8 Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut chip8 = Chip8::new(buf);

    'running: loop {
        chip8.emulate_cycle();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: key,
                    ..
                } => {
                    let keycode = key.unwrap();
                    if keycode == Keycode::Escape {
                        break 'running;
                    } else {
                        for (i, &v) in KEYS.iter().enumerate() {
                            if keycode == v {
                                chip8.key[i] = 1;
                            }
                        }
                    }
                },
                Event::KeyUp {
                    keycode: key,
                    ..
                } => {
                    let keycode = key.unwrap();
                    for (i, &v) in KEYS.iter().enumerate() {
                        if keycode == v {
                            chip8.key[i] = 0;
                        }
                    }
                },
                _ => {}
            }
        }

        if chip8.draw_flag {
            chip8.draw_flag = false;

            for (y, row) in chip8.gfx.iter().enumerate() {
                for (x, &col) in row.iter().enumerate() {
                    let x = (x as u32) * SCALE;
                    let y = (y as u32) * SCALE;

                    let color = if col == 1 {
                        Color::RGB(255, 255, 255)
                    } else {
                        Color::RGB(0, 0, 0)
                    };

                    canvas.set_draw_color(color);
                    canvas.fill_rect(Rect::new(x as i32, y as i32, SCALE, SCALE))?;
                }
            }
            canvas.present();
        }

        sleep(WAIT_TIME);
    }
    Ok(())
}