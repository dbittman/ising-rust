#![feature(rand)]
#![feature(core)] 
extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
use piston::window::WindowSettings;
use std::thread;
use std::rand;
use std::cell::RefCell;
use std::rand::Rng;
use std::num::Float;
use piston::event::{
    events,
    RenderEvent,
};
use sdl2_window::Sdl2Window as Window;
use opengl_graphics::{ Gl, OpenGL };

const COLOR_UP:[f32; 4] = [1.0, 0.0, 0.0, 1.0];
const COLOR_DOWN:[f32; 4] = [0.0, 0.0, 1.0, 1.0];
const temperature: f64 = 2.00;

const WINDOWSIZE: u32 = 1000;
const SIZE: usize = 300;
const BLOCKSIZE: f64 = (WINDOWSIZE as f64 / SIZE as f64);
fn get_rand() -> f64 {
    rand::thread_rng().gen_range(0.0, 1.0)
}

fn energy(s: [[i8; SIZE]; SIZE], i: usize, j: usize) -> i8 {
    let top = match i {
        0 => s[SIZE - 1][j],
        _ => s[i-1][j]
    };
    let bottom = match i + 1 {
        SIZE => s[0][j],
        _    => s[i+1][j]
    };
    let left = match j {
        0 => s[i][SIZE-1],
        _ => s[i][j-1]
    };
    let right = match j + 1 {
        SIZE => s[i][0],
        _    => s[i][j+1]
    };
    -s[i][j] * (top + bottom + left + right)
}

fn delta_u(s: [[i8; SIZE]; SIZE], i: usize, j: usize) -> i8 {
    -2 * energy(s, i, j)
}

fn do_iter(s: &mut [[i8; SIZE]; SIZE], i: usize, j: usize, __energy: f64) -> f64 {
    let mut energy = __energy;
    let ediff = delta_u(*s, i, j);

    if ediff <= 0 {
        s[i][j] = -s[i][j];
        energy += ediff as f64;
    } else {
        if get_rand() < (-ediff as f64 / temperature).exp() {
            s[i][j] = -s[i][j];
            energy += ediff as f64;
        }
    }
    return energy;
}

static mut state: [[i8; SIZE]; SIZE] = [[0i8; SIZE]; SIZE];

fn main() {
    // Create an SDL window.
    let window = Window::new(
        OpenGL::_3_2,
        WindowSettings {
            title: "Ising".to_string(),
            size: [WINDOWSIZE, WINDOWSIZE],
            ..WindowSettings::default()
        });
    let window = RefCell::new(window);
    // Create a new game and run it.
    let mut gl = Gl::new(OpenGL::_3_2);

    for i in range(0, SIZE) {
        for j in range(0, SIZE) {
            unsafe {
                state[i][j] = match get_rand() > 0.5 {
                    true  => 1,
                    false => -1
                }
            }
        }
    }

    thread::spawn(move || {
        let mut energy = 1.0;
        loop {
            let i = (get_rand() * SIZE as f64).floor() as usize;
            let j = (get_rand() * SIZE as f64).floor() as usize;

            let tmp = do_iter(unsafe {&mut state}, i, j, energy);
            energy += tmp;
        }
    });

    for e in events(&window) {
        if let Some(r) = e.render_args() {
            gl.draw([0, 0, r.width as i32, r.height as i32], |_, gl| {
                graphics::clear([0.0; 4], gl);
                for i in range(0, SIZE) {
                    for j in range(0, SIZE) {
                        let col = match unsafe { state[i][j] } {
                            1 => COLOR_UP,
                            _ => COLOR_DOWN
                        };
                        let square = graphics::rectangle::square(i as f64 * BLOCKSIZE,
                                                                 j as f64 * BLOCKSIZE, 10.0);
                        let context = &graphics::Context::abs(WINDOWSIZE as f64,
                                                              WINDOWSIZE as f64);
                        graphics::rectangle(col, square, context.transform, gl);
                    }
                }
            });
        }
    }
}

