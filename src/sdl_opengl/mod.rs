use sdl2;
use gl;
use time;
use game::{GameState, GameInput, GameStepResult};
use std::collections::HashSet;
use self::renderer::Renderer;

mod renderer;

pub struct Game {
    window: sdl2::video::Window,
    _context: sdl2::video::GLContext,
    renderer: Renderer,
    state: GameState
}

enum SDLEventLoopResult {
    HasInput(SDLInput),
    Exit
}

#[deriving(Clone)]
struct SDLInput {
    /// sdl2::scancode::ScanCode doesn't implement Clone, so we need to store an integer representation
    keyboard: HashSet<uint>,
    mouse: Option<(sdl2::mouse::MouseState, int, int)>,
    mouse_drag_from: Option<(sdl2::mouse::MouseState, int, int)>,
    mouse_in_focus: bool
}
impl SDLInput {
    pub fn new() -> SDLInput {
        SDLInput {
            keyboard: HashSet::new(),
            mouse: None,
            mouse_drag_from: None,
            mouse_in_focus: false
        }
    }

    pub fn is_mouse_button_down(&self, button: sdl2::mouse::MouseState) -> bool {
        match self.mouse {
            Some((state, _, _)) => state.intersects(button),
            None => false
        }
    }

    pub fn is_mouse_button_newly_down(&self, old: &SDLInput, button: sdl2::mouse::MouseState) -> bool {
        !old.is_mouse_button_down(button) && self.is_mouse_button_down(button)
    }

    pub fn is_scancode_down(&self, scancode: sdl2::scancode::ScanCode) -> bool {
        let scancode_int = scancode.to_uint().unwrap();
        self.keyboard.contains(&scancode_int)
    }

    pub fn is_scancode_newly_down(&self, old: &SDLInput, scancode: sdl2::scancode::ScanCode) -> bool {
        !old.is_scancode_down(scancode) && self.is_scancode_down(scancode)
    }
}

fn solve_input(old: &SDLInput, new: &SDLInput, viewport: (i32, i32)) -> GameInput {
    let explode = new.is_scancode_newly_down(old, sdl2::scancode::SpaceScanCode);
    let explode_subcube = new.is_mouse_button_down(sdl2::mouse::LEFTMOUSESTATE);
    let reset = new.is_mouse_button_newly_down(old, sdl2::mouse::RIGHTMOUSESTATE);
    let screen_pointer = match new.mouse_in_focus {
        true => match new.mouse {
            Some((_, x, y)) => Some((x as i32, y as i32)),
            None => None
        },
        false => None
    };

    let pointer = match viewport {
        (width, height) => {
            match screen_pointer {
                Some((x, y)) => Some(((x as f32 / width as f32 - 0.5)*2.0, -(y as f32 / height as f32 - 0.5)*2.0)),
                None => None
            }
        }
    };

    GameInput {
        explode: explode,
        explode_subcube: explode_subcube,
        reset: reset,
        pointer: pointer,
        rotate_view: (0.0, 0.0)
    }
}

impl Game {
    pub fn new(width: int, height: int) -> Result<Game, String> {
        sdl2::init(sdl2::INIT_VIDEO);

        sdl2::video::gl_set_attribute(sdl2::video::GLContextMajorVersion, 3);
        sdl2::video::gl_set_attribute(sdl2::video::GLContextMinorVersion, 0);
        sdl2::video::gl_set_attribute(sdl2::video::GLDepthSize, 24);
        sdl2::video::gl_set_attribute(sdl2::video::GLDoubleBuffer, 1);
        sdl2::video::gl_set_attribute(
            sdl2::video::GLContextProfileMask,
            sdl2::video::ll::SDL_GL_CONTEXT_PROFILE_CORE as int);

        let window = match sdl2::video::Window::new("Rust cubes demo", sdl2::video::PosCentered, sdl2::video::PosCentered, width, height, sdl2::video::OPENGL | sdl2::video::SHOWN | sdl2::video::RESIZABLE) {
            Ok(window) => window,
            Err(err) => return Err(format!("failed to create window: {}", err))
        };

        let context = match window.gl_create_context() {
            Ok(context) => context,
            Err(err) => return Err(format!("failed to create context: {}", err))
        };

        // Initialize the OpenGL function pointers
        gl::load_with(|s: &str| unsafe {
            use std;
            match sdl2::video::gl_get_proc_address(s) {
                Some(ptr) => std::mem::transmute(ptr),
                None => std::ptr::null()
            }
        });

        let renderer = try!(Renderer::new());
        let state = GameState::new();

        Ok(Game {
            window: window,
            _context: context,
            renderer: renderer,
            state: state
        })
    }

    fn frame_limit(&self) -> Option<u32> {
        // Twice the rate of a typical computer monitor
        // Some(120)
        None
    }

    fn event_loop(&self) -> SDLEventLoopResult {
        'event: loop {
            match sdl2::event::poll_event() {
                sdl2::event::QuitEvent(_) => { return Exit; },
                sdl2::event::KeyDownEvent(_, _, key, _, _) => {
                    if key == sdl2::keycode::EscapeKey {
                        return Exit;
                    }
                },
                sdl2::event::NoEvent => { break 'event; },
                _ => ()
            }
        }

        let mouse = sdl2::mouse::get_mouse_state();
        let keys = sdl2::keyboard::get_keyboard_state();

        let mouse_in_focus = match sdl2::mouse::get_mouse_focus() {
            Some(_window) => true,
            None => false
        };

        let mut keyboard = HashSet::new();
        for (scancode, pressed) in keys.iter() {
            if *pressed {
                keyboard.insert(ToPrimitive::to_uint(scancode).unwrap());
            }
        }

        HasInput(SDLInput {
            keyboard: keyboard,
            mouse: Some(mouse),
            mouse_in_focus: mouse_in_focus,
            mouse_drag_from: None
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        let step_interval: f64 = 1.0/(GameState::steps_per_second() as f64);

        struct Frame {
            time: f64,
            viewport: (i32,i32)
        }

        struct Step {
            input: SDLInput,
            result: GameStepResult
        }

        // Define an initial "last frame".
        let mut last_frame = Frame {
            time: time::precise_time_s(),
            viewport: self.get_viewport()
        };

        let mut last_step = Step {
            input: SDLInput::new(),
            result: self.state.step(last_frame.viewport, &GameInput::new())
        };

        let mut step_error: f64 = 0.0;

        let mut fps_meter = FPSMeter::new(1.0);
        let mut fps_meter_change = ValueOnChange::new();

        // Run subsequent frames in a loop
        // The loop always has a "last frame" to refer to
        'main: loop {
            let input = match self.event_loop() {
                HasInput(input) => input,
                Exit => break 'main
            };

            let current_frame = Frame {
                time: time::precise_time_s(),
                viewport: self.get_viewport()
            };

            let delta: f64 = current_frame.time - last_frame.time;

            step_error += delta;

            while step_error >= step_interval {
                let game_input = solve_input(&last_step.input, &input, current_frame.viewport);
                let result = self.state.step(current_frame.viewport, &game_input);
                step_error -= step_interval;

                last_step = Step {
                    input: input.clone(),
                    result: result
                };
            }

            self.renderer.render(&self.state, &last_step.result, current_frame.viewport);

            self.window.gl_swap_window();

            match self.frame_limit() {
                Some(fps) => {
                    let d = time::precise_time_s() - current_frame.time;
                    let ms = 1000/fps as int - (d*1000.0) as int;
                    if ms > 0 {
                        sdl2::timer::delay(ms as uint)
                    }
                },
                None => ()
            }

            // Update FPS
            fps_meter.meter_frame();

            // Show FPS when it changes
            match fps_meter_change.value(fps_meter.get_fps()) {
                Some(fps) => match fps {
                    Some(fps) => println!("{} FPS", fps),
                    None => ()  // no FPS recorded
                },
                None => ()      //no change
            }

            last_frame = current_frame;
        }

        Ok(())
    }

    fn get_viewport(&self) -> (i32,i32) {
        match self.window.get_size() {
            (w, h) => (w as i32, h as i32)
        }
    }
}

struct FPSMeter {
    interval: f64,
    time_measure_begin: f64,
    frames_since: u32,
    last_fps: Option<f64>
}
impl FPSMeter {
    pub fn new(interval: f64) -> FPSMeter {
        FPSMeter {
            interval: interval,
            time_measure_begin: time::precise_time_s(),
            frames_since: 0,
            last_fps: None
        }
    }
    pub fn meter_frame(&mut self) {
        let time = time::precise_time_s();
        let delta = time - self.time_measure_begin;

        if delta >= self.interval {
            self.last_fps = Some(self.frames_since as f64 / self.interval);
            self.time_measure_begin += self.interval;
            self.frames_since = 0;
        }
        self.frames_since += 1;
    }
    pub fn get_fps(&self) -> Option<f64> { self.last_fps }
}

struct ValueOnChange<T> {
    old: Option<T>
}
impl<T: Copy + PartialEq> ValueOnChange<T> {
    pub fn new() -> ValueOnChange<T> {
        ValueOnChange { old: None }
    }

    pub fn value(&mut self, value: T) -> Option<T> {
        if Some(value) != self.old {
            // changed
            self.old = Some(value);
            Some(value)
        } else {
            None
        }
    }
}
