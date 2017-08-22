extern crate sdl2;
extern crate rand;

use rand::Rng;
use std::mem;
use std::process;
use std::collections::{ HashMap, HashSet };
use std::env;
use std::rc::Rc;
use std::borrow::Borrow;
use std::hash::Hash;
use std::fs::File;
use std::io::prelude::*;
use std::io::{ BufReader,SeekFrom, Read, Cursor };
use std::path::Path;

use sdl2::image::{LoadTexture, INIT_PNG, INIT_JPG};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::video::Window;
use sdl2::rect::{Point, Rect};
use sdl2::render::{ TextureCreator, Texture, Canvas };

const  chip8_fontset: [usize;80]  =
[
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

struct Chip8 {
    draw_flag: bool,
    gfx: Vec<usize>,
    key : Vec<usize>,
    pc: usize,
    opcode : usize,
    I : usize,
    sp : usize,
    V: Vec<usize>,
    stack: Vec<usize>,
    memory: Vec<usize>,
    delay_timer: usize,
    sound_timer: usize,
}

impl Chip8 {
    fn init(&mut self) {

          self.pc = 0x200;
          self.opcode  = 0;
          self.I = 0;
          self.sp = 0;
          self.gfx = Vec::with_capacity(2048);
          self.V = Vec::with_capacity(16);
          self.stack = Vec::with_capacity(16);
          self.delay_timer = 0;
          self.sound_timer = 0;
          self.key = Vec::with_capacity(16);
          self.draw_flag = false;
          self.memory = Vec::with_capacity(4096);

        for i in 0..2048 {
            self.gfx[i]  = 0;
        }

        for i in 0..16 {
            self.stack[i]  = 0;
        }

        for i in 0..16 {
            self.key[i]  =  0;
        }

        for i in 0..16 {
            self.V[i] = 0;
        }

        for i in 0..4096 {
            self.memory[i] = 0;
        }

        for i in 0..80 {
            self.memory[i] = chip8_fontset[i];
        }

        self.delay_timer = 0;
        self.sound_timer = 0;
        self.draw_flag  = true;

    }

    fn emulate_cycle(&mut self) {
        self.opcode  = self.memory[self.pc as usize] << 8 | self.memory[ self.pc as usize + 1];

        match  self.opcode & 0xF000 {
        0x0000 => {
            for i in 0..2048 {
                self.draw_flag = true;
                self.pc += 2;
            }
        },
        0x000E => {
            self.sp -= 1;
            self.pc = self.stack[self.sp as usize];
            self.pc +=2;
        },
        0x1000 => {
            self.pc = self.opcode  & 0x0FFF;
        },
        0x2000 => {
            self.stack[self.sp as usize] = self.pc;
            self.pc += 1;
            self.pc = self.opcode  & 0x0fff;
        },
        0x3000 => {
            if self.V[(self.opcode as usize as usize & 0x0F00) >> 8] == (self.opcode  & 0x00FF) {
                self.pc += 4;
            } else {
                self.pc += 2;
            }
        },
        0x4000 => {
            if self.V[(self.opcode as usize as usize & 0x0F00) >> 8] != (self.opcode & 0x00FF) {
                self.pc += 4;
            } else {
                self.pc += 2;
            }
        }
        0x5000 => {
            if self.V[(self.opcode as usize as usize & 0x0F00) >> 8] == self.V[(self.opcode as usize as usize & 0x00F0) >> 4] {
                self.pc += 4;
            } else {
                self.pc += 2;
        }
    },
        0x6000 => {
            self.V[(self.opcode as usize as usize & 0x0f00)>> 8] = self.opcode & 0x00FF;
            self.pc += 2;
        },
        0x7000 => {
            self.V[(self.opcode as usize & 0x0F00) >> 8 ] += self.opcode & 0x00FF;
            self.pc += 2;
        },
        0x8000 => {
            match self.opcode as usize & 0x000F {
                0x0000 => {
                    self.V[(self.opcode as usize & 0x0F00) >> 8] = self.V[(self.opcode as usize & 0x00F0) >> 4];
                    self.pc += 2;
                },
                0x0001 => {
                    self.V[(self.opcode as usize & 0xF00) >> 8] |= self.V[(self.opcode as usize & 0x00F0) >> 4];
                    self.pc += 2;
                },
                0x0002 => {
                    self.V[(self.opcode as usize & 0x0F00) >> 8] &= self.V[(self.opcode as usize & 0x00F0) >> 4];
                    self.pc += 2;
                },
                0x0003 => {
                    self.V[(self.opcode as usize & 0x0F00) >> 8] ^= self.V[(self.opcode as usize & 0x00F0) >> 4];
                    self.pc += 2;
                },
                0x0004 => {
                    if self.V[(self.opcode as usize & 0x00F0) >> 4] > (0xFF - self.V[(self.opcode as usize & 0x0F00) >> 8]){
                        self.V[0xF] = 1;
                    } else {
                        self.V[0xF] = 0;
                        self.V[(self.opcode as usize & 0x0F00) >> 8] += self.V[(self.opcode as usize & 0x00F0) >> 4];
                        self.pc += 2;
                    }
                },
                0x0005 => {
                    if self.V[(self.opcode as usize & 0x00F0) >> 8] > self.V[(self.opcode as usize & 0x0F00) >> 8] {
                        self.V[0xF] = 0;
                    } else {
                        self.V[0xF] = 1;
                        self.V[(self.opcode as usize & 0x0F00) >> 8] -= self.V[(self.opcode as usize & 0x0F00) >> 8];
                        self.pc += 2;
                    }
                },
                0x0006 => {
                    self.V[0xF] = self.V[(self.opcode as usize & 0x0F00) >> 8] & 0x1;
                    self.V[(self.opcode as usize & 0x0F00) >> 8] >>=1;
                    self.pc += 2;
                },
                0x0007 => {
                    if self.V[(self.opcode as usize & 0x0F00) >> 8] > self.V[(self.opcode as usize & 0x00F0) >> 4] {
                        self.V[0xF] = 0;
                    } else {
                        self.V[0xF] = 1;
                        self.V[(self.opcode as usize & 0x0F00) >> 8] = self.V[(self.opcode as usize & 0x00F0) >> 4] - self.V[(self.opcode as usize & 0x0F00) >> 8];
                        self.pc += 2;
                    }
                },
                0x000E => {
                    self.V[0xF] = self.V[(self.opcode as usize & 0x0F00) >> 8] >> 7;
                    self.V[(self.opcode as usize  & 0x0F00) >> 8] <<=1;
                    self.pc += 2;
                },
                _ => println!("Unkown opcode:[0x8000]: {}\n",self.opcode as usize),
                }
            },
            0x9000 => {
                if self.V[(self.opcode as usize & 0x0F00) >> 8] != self.V[(self.opcode as usize & 0x00F0) >> 4]{
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0xA000 => {
                self.I = self.opcode  & 0x0FFF;
                self.pc += 2;
            },
            0xB000 => {
                self.pc = (self.opcode  & 0x0FFF) + self.V[0];
            },
            0xC000 => {
                self.V[(self.opcode as usize  & 0x0F00) >> 8] = (rand::thread_rng().gen_range(1, 101) % 0xFF) & (self.opcode & 0x00FF);
                self.pc += 2;
            },
            0xD000 => {
                    let x: usize= self.V[(self.opcode as usize &  0x0F00) >> 8] as usize;
                    let y: usize= self.V[(self.opcode as usize & 0x00F0)>> 4] as usize;
                    let height:usize = self.opcode &  0x000F;
                    self.V[0xF] = 0;
                    for yline in 0..height as usize {
                        let pixel:usize = self.memory[self.I as usize + yline];
                        for xline in 0..8 as usize{
                            if (pixel & (0x80 >> xline)) != 0 {
                                if self.gfx[(x + xline + ((y + yline) * 64))as usize] == 1 {
                                    self.V[0xF] = 1;
                                }
                                self.gfx[x + xline + ((y + yline) * 64 )] ^= 1;
                            }
                        }
                    }
                    self.draw_flag = true;
                    self.pc += 2;
            },
            0xE000 => {
                match self.opcode as usize & 0x00FF {
                    0x009E => {
                        if self.key[self.V[(self.opcode as usize & 0x0F00) >> 8]as usize] != 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    0x00A1 => {
                        if self.key[self.V [(self.opcode as usize & 0x0F00)>> 8] as usize] == 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    _ => println!("Unknown opcode[0xE000]: {}", self.opcode as usize),
                }
            },
            0xF000 => {
                match self.opcode as usize & 0x00FF {
                    0x0007 => {
                        self.V[(self.opcode as usize & 0x0F00) >> 8] = self.delay_timer;
                        self.pc += 2;
                    },
                    0x000A => {
                        let mut key_press: bool = false;
                        for i in 0..16 {
                            if self.key[i as usize] != 0 {
                                self.V[(self.opcode as usize & 0x0F00) >> 8] = i;
                                key_press = true;
                            }
                        }
                        if !key_press {
                            return;
                        }
                        self.pc+= 2;
                    },
                    0x0015 => {
                        self.delay_timer = self.V[(self.opcode as usize & 0x0F00) >> 8];
                        self.pc += 2;
                    },
                    0x0018 => {
                        self.sound_timer = self.V[(self.opcode as usize & 0x0F00) >> 8];
                        self.pc += 2;
                    },
                    0x001E => {
                        if self.I + self.V[(self.opcode as usize & 0x0F00) >> 8] > 0xFFF {
                            self.V[0xF] = 1;
                        } else {
                            self.V[0xF] = 0;
                            self.I += self.V[(self.opcode as usize & 0x0F00) >> 8];
                            self.pc += 2;
                        }
                    },
                    0x0033 => {
                        self.memory[self.I as usize] = self.V[(self.opcode as usize & 0x0F00) >> 8] / 100;
                        self.memory[self.I as usize + 1] = (self.V[(self.opcode as usize & 0x0F00) >> 8] / 10) % 10;
                        self.memory[self.I as usize+ 2] = (self.V[(self.opcode as usize & 0x0F00) >> 8] % 100) % 10;
                        self.pc += 2;
                    },
                    0x0055 => {
                        for i in 0..((self.opcode as usize & 0x0F00) as usize >> 8) {
                            self.V[self.I as usize + i as usize]  = self.memory[self.I as usize + 1];
                            self.I += (self.opcode & 0x0F00) >> 8  + 1;
                            self.pc += 2;
                        }
                    },
                    _ => println!("Unknown opcode [0xF000]: {}", self.opcode as usize)

                }
            },
            _ => println!("Unknown opcode: 0x{}\n",self.opcode as usize),
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP\n");
            self.sound_timer -= 1;
            }
        }
    }

    fn debug_render(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                if self.gfx[(y*64) + x] == 0 {
                    println!("0");
                } else {
                    println!("  ");
                }
            }
                println!("\n");
        }
                println!("\n");
    }

    fn load_application<P: AsRef<Path>>(&mut self, file_path: P)-> Result<(), (String)>
                                    where P:std::fmt::Display  {
        self.init();
        println!("Loading: {}\n",file_path);
        //FIXME: Nešto je trulo u državi Danskoj.

        let mut file = try!(File::open(file_path).map_err(|e|e.to_string()));
        let mut reader = BufReader::new(file);
        let mut buffer: Vec<u8> = Vec::with_capacity(2*4096);
        reader.read(&mut buffer);
        let size = reader.bytes().count();

        if (4096 - 512) > size {
                for i in 0..size {
                    self.memory[i + 512] = buffer[i] as usize;
                }
        } else {
            println!("Error: ROM to big for memory");
        }

        Ok(())
    }

}
const SCREEN_WIDTH:u32 = 64;
const SCREEN_HEIGHT:u32= 32;

fn init_sdl() ->  (Canvas<Window>, sdl2::EventPump) {
    let sdl_context = sdl2::init ().ok ().expect ("Could not initialize SDL2");
    let video_subsystem  = sdl_context.video ().ok ().expect ("Could not init video_subsystem");

    let display_width = SCREEN_WIDTH * 10;
    let display_height = SCREEN_HEIGHT * 10;
    let window = video_subsystem.window ("Chip8", 800, 600)
        .position_centered ()
        .opengl ()
        .build ()
        .unwrap ();

    let canvas = window.into_canvas ()
        .present_vsync ()
        .build ()
        .unwrap ();

    let event_pump = sdl_context.event_pump ().unwrap ();

    (canvas, event_pump)
}

fn main() {

}

type TextureManager<'l, T> = ResourceManager<'l, String, Texture<'l>, TextureCreator<T>>;

pub struct ResourceManager<'l, K, R, L>
where K: Hash + Eq,
      L: 'l + ResourceLoader<'l, R>
{
    loader: &'l L,
    cache: HashMap<K, Rc<R>>,
}

impl<'l,K, R, L> ResourceManager<'l, K, R, L>
where K: Hash + Eq,
      L:  ResourceLoader<'l, R>
{
    pub fn new(loader: &'l L) -> Self {
        ResourceManager {
            cache: HashMap::new(),
            loader: loader,
        }
    }

    pub fn load<D>(&mut self, details: &D) -> Result<Rc<R>, String>
        where L: ResourceLoader<'l, R, Args=D>,
              D: Eq + Hash + ?Sized,
              K: Borrow<D> + for<'a>From<&'a D>
              {
                  self.cache
                      .get(details)
                      .cloned()
                      .map_or_else(|| {
                          let resource = Rc::new(self.loader.load(details)?);
                          self.cache.insert(details.into(), resource.clone());
                          Ok(resource)
                      },
                      Ok)
              }
}

impl<'l, T>ResourceLoader<'l, Texture<'l>> for TextureCreator<T> {
    type Args = str;
    fn load(&'l self, path: &str) -> Result<Texture, String> {
        println!("LOADED A TEXTURE");
        self.load_texture(path)
    }

}

pub trait ResourceLoader<'l, R> {
    type Args: ?Sized;
    fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}
