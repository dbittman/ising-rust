#![feature(rand)]
#![feature(core)] 
extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
use graphics::vecmath::Matrix2d;
use piston::window::WindowSettings;
use std::thread;
use std::rand;
use std::cell::RefCell;
use std::rand::Rng;
use std::num::Float;
use std::sync::atomic::{AtomicUsize, Ordering, AtomicIsize};
use std::sync::{Arc, Mutex};
use piston::input::Button;
use piston::input::keyboard::Key;
use piston::event::{
    events,
    PressEvent,
    ReleaseEvent,
    RenderEvent,
};
use sdl2_window::Sdl2Window as Window;
use opengl_graphics::{ Gl, OpenGL, Texture };

const COLOR_UP:[f32; 4] = [0.8, 0.2, 0.2, 1.0];
const COLOR_DOWN:[f32; 4] = [0.2, 0.2, 0.8, 1.0];
static mut temperature: f64 = 2.0;

const WINDOWSIZE: u32 = 800;
const SIZE: usize = 200;
const BLOCKSIZE: f64 = (WINDOWSIZE as f64 / SIZE as f64);
fn get_rand() -> f64 {
    rand::thread_rng().gen_range(0.0, 1.0)
}

fn calc_energy(s: [[i8; SIZE]; SIZE], i: usize, j: usize) -> i8 {
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
    -2 * calc_energy(s, i, j)
}

fn do_iter(s: &mut [[i8; SIZE]; SIZE], i: usize, j: usize) -> i8 {
    let mut newenergy = 0;
    let ediff = delta_u(*s, i, j);

    if ediff <= 0 {
        s[i][j] = -s[i][j];
        newenergy = ediff;
    } else {
        if get_rand() < (-ediff as f64 / unsafe { temperature }).exp() {
            s[i][j] = -s[i][j];
            newenergy = ediff;
        }
    }
    return newenergy;
}

static mut state: [[i8; SIZE]; SIZE] = [[0i8; SIZE]; SIZE];

fn main() {
    // Create an SDL window.
    let window = Window::new(
        OpenGL::_3_2,
        WindowSettings {
            title: "Ising".to_string(),
            size: [WINDOWSIZE, WINDOWSIZE + 100],
            ..WindowSettings::default()
        });
    let window = RefCell::new(window);
    // Create a new game and run it.
    let mut gl = Gl::new(OpenGL::_3_2);

    for i in range(0, SIZE) {
        for j in range(0, SIZE) {
            unsafe {
                state[i][j] = match get_rand() < 0.5 {
                    true  => 1,
                    false => -1
                }
            }
        }
    }

    let mut initial_energy: isize = 1;
    for i in range(0, SIZE) {
        for j in range(0, SIZE) {
            print!("calculating initial energy state: {} / {} ({}%)\r",
            i*SIZE + j + 1, SIZE * SIZE, ((i * SIZE + j + 1) * 100) / (SIZE * SIZE));
            initial_energy += calc_energy(unsafe { * &mut state }, i, j) as isize;
        }
    }
    println!("\ninitial energy: {}", initial_energy);

    let iters = Arc::new(AtomicUsize::new(0));
    let ubar_value: isize = initial_energy;
    let ubar  = Arc::new(AtomicIsize::new(ubar_value));
    let energy  = Arc::new(AtomicIsize::new(initial_energy));

    for threadnum in range(0, 2) {
        let iters = iters.clone();
        let ubar = ubar.clone();
        let energy = energy.clone();
        thread::spawn(move || {
            loop {
                let iter = iters.fetch_add(1, Ordering::Relaxed) + 1;
                let i = (get_rand() * SIZE as f64).floor() as usize;
                let j = (get_rand() * SIZE as f64).floor() as usize;

                let energy = energy.fetch_add(
                    do_iter(unsafe {&mut state}, i, j) as isize, Ordering::Relaxed);
                let u = ubar.fetch_add(energy, Ordering::Relaxed);
                if threadnum == 0 && iter % 100 == 0 {
                    print!("iteration {} ({} per cell) : average energy per cell {}\r", iter, iter / (SIZE * SIZE),
                    u as f64 / ((iter * SIZE * SIZE + 1) as f64));
                }
            }
        });
    }

    for e in events(&window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            unsafe {
                temperature += match(key) {
                    Key::Up => 0.1,
                    Key::Down => -0.1,
                    Key::Right => 0.01,
                    Key::Left => -0.01,
                    _ => 0.0
                };
                if temperature < 0.0 {
                    temperature = 0.0;
                }
                println!("\nNew temperature: {}", unsafe { temperature });
            }
        };

        if let Some(r) = e.render_args() {
            gl.draw([0, 100, r.width as i32, (r.height - 100) as u32 as i32], |_, gl| {
                graphics::clear([0.0; 4], gl);
                for i in range(0, SIZE) {
                    for j in range(0, SIZE) {
                        let col = match unsafe { state[i][j] } {
                            1 => COLOR_UP,
                            _ => COLOR_DOWN
                        };
                        let square = graphics::rectangle::square(i as f64 * BLOCKSIZE,
                                                                 j as f64 * BLOCKSIZE, BLOCKSIZE);
                        let context = &graphics::Context::abs(WINDOWSIZE as f64,
                                                              WINDOWSIZE as f64);
                        graphics::rectangle(col, square, context.transform, gl);
                    }
                }
            });
        }
    }
}

