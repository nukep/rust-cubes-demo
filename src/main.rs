extern crate cgmath;
extern crate game;
extern crate gl;
extern crate sdl2;
extern crate time;
extern crate util;

mod sdl_opengl;
#[allow(dead_code)]
mod opengl_util;

pub fn main() {
    let mut sdl_game = match sdl_opengl::Game::new(800, 600) {
        Ok(sdl_game) => sdl_game,
        Err(msg) => panic!("sdl_opengl::Game::new: {}", msg)
    };

    match sdl_game.run() {
        Ok(()) => (),
        Err(msg) => panic!("sdl_opengl::Game::run: {}", msg)
    }
}
