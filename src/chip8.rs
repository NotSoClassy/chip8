const CODE_OFFSET: usize = 0x200; // 512
const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const REG_SIZE: usize = 16;

pub const CHIP8_HEIGHT: usize = 32;
pub const CHIP8_WIDTH: usize = 64;

const SPRITES: [u8; 80] = [
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

use rand::prelude::*;

pub struct Chip8 {
  pub sound_timer: u8,
  pub delay_timer: u8,
  pub draw_flag: bool,
  pub memory: [u8; MEMORY_SIZE],
  pub stack: [u16; STACK_SIZE],
  pub i_reg: u16,
  pub key: [u8; 16],
  pub rng: rand::rngs::ThreadRng,
  pub gfx: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
  pub pc: u16,
  pub sp: u16,
  pub v: [u8; REG_SIZE]
}

fn read(memory: [u8; MEMORY_SIZE], index: u16) -> u16 {
  ((memory[index as usize] as u16) << 8)
    | (memory[(index + 1) as usize] as u16)
}

impl Chip8 {
  pub fn new(data: Vec<u8>) -> Self {

    let mut mem = [0; MEMORY_SIZE];

    for (i, v) in SPRITES.iter().enumerate() {
      mem[i] = v.clone();
    }

    for (i, v) in data.iter().enumerate() {
      mem[CODE_OFFSET + i] = v.clone();
    }

    let rng = thread_rng();

    Chip8 {
      sound_timer: 0,
      delay_timer: 0,
      draw_flag: false,
      memory: mem,
      stack: [0; STACK_SIZE],
      i_reg: 0,
      key: [0; 16],
      rng: rng,
      gfx: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
      pc: CODE_OFFSET as u16,
      sp: 0,
      v: [0; REG_SIZE]
    }
  }
  pub fn emulate_cycle(&mut self) {
    let instr = read(self.memory, self.pc);
    self.pc += 2;
    // decode
    let opcode = (instr >> 12) & 0xF; // X000
    let x = (instr >> 8) & 0xF; // 0X00
    let y = (instr >> 4) & 0xF; // 00X0
    let n = instr & 0xF; // 000X
    let nn = instr & 0xFF; // 00XX
    let nnn = instr & 0xFFF; // 0XXX

    match opcode {
      0x0 => {
        match n {
          0x0 => {
            for y in 0..CHIP8_HEIGHT {
              for x in 0..CHIP8_WIDTH {
                self.gfx[y][x] = 0;
              }
           }
          },
          0xE => {
            self.sp -= 1;
            self.pc = self.stack[self.sp as usize] + 2;
          }
          _ => { panic!("Unknown opcode {:#x}", instr); }
        }
      },
      0x1 => {
        self.pc = nnn;
      },
      0x2 => {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
      },
      0x3 => {
        if self.v[x as usize] == nn as u8 {
          self.pc += 2;
        }
      },
      0x4 => {
        if self.v[x as usize] != nn as u8 {
          self.pc += 2;
        }
      },
      0x5 => {
        if self.v[x as usize] == self.v[y as usize] {
          self.pc += 2;
        }
      },
      0x6 => {
        self.v[x as usize] = nn as u8;
      },
      0x7 => {
        let res = self.v[x as usize] as u16 + nn;
        self.v[x as usize] = res as u8;
      },
      0x8 => {
        match n {
          0x0 => {
            self.v[x as usize] = self.v[y as usize];
          },
          0x1 => {
            self.v[x as usize] |= self.v[y as usize];
          },
          0x2 => {
            self.v[x as usize] &= self.v[y as usize];
          },
          0x3 => {
            self.v[x as usize] ^= self.v[y as usize];
          },
          0x4 => {
            let res = (self.v[x as usize] as u16) + (self.v[y as usize] as u16);
            self.v[0xF] = (res > 255) as u8;
            self.v[x as usize] = res as u8;
          },
          0x5 => {
            self.v[0xF] = (self.v[x as usize] > self.v[y as usize]) as u8;
            self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
          },
          0x6 => {
            self.v[0xF] = self.v[x as usize] & 0x1;
            self.v[x as usize] >>= 1;
          },
          0x7 => {
            if self.v[x as usize] > self.v[y as usize] {
              self.v[0xF] = 0;
            } else {
              self.v[0xF] = 1;
            }
            self.v[x as usize] = self.v[y as usize] - self.v[x as usize];
          },
          0xE => {
            self.v[0xF] = self.v[x as usize] >> 7;
            self.v[x as usize] <<= 1;
          },
          _ => { panic!("Unknown opcode {:#x}", instr); }
        }
      },
      0x9 => {
        if self.v[x as usize] != self.v[y as usize] {
          self.pc += 2;
        }
      },
      0xA => {
        self.i_reg = nnn;
      },
      0xB => {
        self.pc = nnn + self.v[0] as u16;
      },
      0xC => {
        self.v[x as usize] = self.rng.gen::<u8>() & nn as u8;
      },
      0xD => {
        self.v[0xF] = 0;

        for byte in 0..n {
          let y = (self.v[y as usize] as usize + (byte as usize)) % CHIP8_HEIGHT;
          for bit in 0..8 {
            let x = (self.v[x as usize] as usize + bit) % CHIP8_WIDTH;
            let color = (self.memory[(self.i_reg + byte) as usize] >> (7 - bit)) & 1;

            self.v[0xF] |= color & self.gfx[y][x];
            self.gfx[y][x] ^= color;
          }
        }
        self.draw_flag = true;
      },
      0xE => {
        match nn {
          0x9E => {
            if self.key[self.v[x as usize] as usize] == 1 {
              self.pc += 2;
            }
          },
          0xA1 => {
            if self.key[self.v[x as usize] as usize] == 0 {
              self.pc += 2;
            }
          },
          _ => { panic!("Unknown opcode {:#x}", instr); }
        }
      },
      0xF => {
        match nn {
          0x07 => {
            self.v[x as usize] = self.delay_timer;
          },
          0x0A => {
            let mut key_pressed = false;

            for (i, &v) in self.key.iter().enumerate() {
              if v == 1 {
                self.v[x as usize] = i as u8;
                key_pressed = true;
              }
            }

            if !key_pressed {
              self.pc -= 2;
              return // try again
            }
          },
          0x15 => {
            self.delay_timer = self.v[x as usize];
          },
          0x18 => {
            self.sound_timer = self.v[x as usize];
          },
          0x1E => {
            self.v[0xF] = (self.i_reg + (self.v[x as usize] as u16) > 0xFFF) as u8;
            self.i_reg += self.v[x as usize] as u16;
          },
          0x29 => {
            self.i_reg = (self.v[x as usize] as u16) * 0x5;
          },
          0x33 => {
            let i = self.i_reg as usize;
            self.memory[i] = self.v[x as usize] / 100;
            self.memory[i + 1] = (self.v[x as usize] / 10) % 10;
            self.memory[i + 2] = self.v[x as usize] % 10;
          },
          0x55 => {
            for i in 0..x {
              self.memory[(self.i_reg + (i as u16)) as usize] = self.v[i as usize];
            }

            self.i_reg += (self.v[x as usize] as u16) + 1;
          },
          0x65 => {
            for i in 0..x {
              self.v[i as usize] = self.memory[(self.i_reg + i as u16) as usize];
            }

            self.i_reg += (self.v[x as usize] as u16) + 1;
          },
          _ => { panic!("Unknown opcode {:#x}", instr); }
        }
      },
      _ => { panic!("Unknown opcode: {:#x}", instr); }
    }

    if self.delay_timer > 0 {
      self.delay_timer -= 1;
    }

    if self.sound_timer > 0 {
      if self.sound_timer == 1 {
        println!("beep");
      }
      self.sound_timer -= 1;
    }
  }
}