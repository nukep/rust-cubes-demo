extern crate cgmath;
extern crate gl;
extern crate sdl2;
extern crate rand;
extern crate num;
extern crate collision;
// extern crate time;

mod sdl_opengl;
#[allow(dead_code)]
mod opengl_util;

pub mod game;
pub mod util;

pub fn main() {
    let mut sdl_game = match sdl_opengl::Game::new(1920, 1080) {
        Ok(sdl_game) => sdl_game,
        Err(msg) => panic!("sdl_opengl::Game::new: {}", msg)
    };

    match sdl_game.run() {
        Ok(()) => (),
        Err(msg) => panic!("sdl_opengl::Game::run: {}", msg)
    }
}
