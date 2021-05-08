extern crate sdl2;
extern crate rand;

use sdl2::keyboard::{ Keycode, Scancode };
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
const WAIT_TIME: Duration = Duration::from_millis(1000 / 500);

/*const KEYS: [Keycode; 16] = [
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
];*/

fn main() -> Result<(), String>{
    let name = env::args().nth(1).expect("expected file");
    let mut file = File::open(name).expect("could not open file");
    let mut buf = Vec::new();

    file.read_to_end(&mut buf).expect("Failure to read file");

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
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
                _ => {}
            }
        }

        let keyboard = sdl2::keyboard::KeyboardState::new(&event_pump);
        chip8.key[1]  = keyboard.is_scancode_pressed(Scancode::Num1);
        chip8.key[2]  = keyboard.is_scancode_pressed(Scancode::Num2);
        chip8.key[3]  = keyboard.is_scancode_pressed(Scancode::Num3);
        chip8.key[12] = keyboard.is_scancode_pressed(Scancode::Num4);

        chip8.key[4]  = keyboard.is_scancode_pressed(Scancode::Q);
        chip8.key[5]  = keyboard.is_scancode_pressed(Scancode::W);
        chip8.key[6]  = keyboard.is_scancode_pressed(Scancode::E);
        chip8.key[13] = keyboard.is_scancode_pressed(Scancode::R);

        chip8.key[7]  = keyboard.is_scancode_pressed(Scancode::A);
        chip8.key[8]  = keyboard.is_scancode_pressed(Scancode::S);
        chip8.key[9]  = keyboard.is_scancode_pressed(Scancode::D);
        chip8.key[14] = keyboard.is_scancode_pressed(Scancode::F);

        chip8.key[10] = keyboard.is_scancode_pressed(Scancode::Z);
        chip8.key[11] = keyboard.is_scancode_pressed(Scancode::X);
        chip8.key[0]  = keyboard.is_scancode_pressed(Scancode::C);
        chip8.key[15] = keyboard.is_scancode_pressed(Scancode::V);

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