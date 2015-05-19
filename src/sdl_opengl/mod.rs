use sdl2;
use gl;
use time;
use game::{GameState, GameInput, GameStepResult};
use std::collections::HashSet;
use self::renderer::Renderer;

mod renderer;

pub struct Game {
    sdl: sdl2::Sdl,
    window: sdl2::video::Window,
    _context: sdl2::video::GLContext,
    renderer: Renderer,
    state: GameState,

    mouse_wheel_absolute: (i32, i32)
}

enum SDLEventLoopResult {
    HasInput(SDLInput),
    Exit
}

#[derive(Clone)]
struct SDLInput {
    /// sdl2::scancode::ScanCode doesn't implement Clone, so we need to store an integer representation
    keyboard: HashSet<sdl2::scancode::ScanCode>,
    mouse: Option<(sdl2::mouse::MouseState, i32, i32)>,
    mouse_in_focus: bool,
    mouse_wheel_absolute: (i32, i32)
}
impl SDLInput {
    pub fn new() -> SDLInput {
        SDLInput {
            keyboard: HashSet::new(),
            mouse: None,
            mouse_in_focus: false,
            mouse_wheel_absolute: (0, 0)
        }
    }

    pub fn is_mouse_button_down(&self, button: sdl2::mouse::Mouse) -> bool {
        match self.mouse {
            Some((state, _, _)) => state.button(button),
            None => false
        }
    }

    pub fn is_mouse_button_newly_down(&self, old: &SDLInput, button: sdl2::mouse::Mouse) -> bool {
        !old.is_mouse_button_down(button) && self.is_mouse_button_down(button)
    }

    pub fn get_mouse_delta(&self, old: &SDLInput, button: sdl2::mouse::Mouse) -> Option<(i32, i32)> {
        match (self.mouse, old.mouse) {
            (Some((n_state, n_x, n_y)), Some((o_state, o_x, o_y))) => {
                if n_state.button(button) && o_state.button(button) {
                    match (n_x - o_x, n_y - o_y) {
                        // A delta of (0, 0) means there was no change
                        (0, 0) => None,
                        delta => Some(delta)
                    }
                } else {
                    None
                }
            },
            _ => None
        }
    }

    pub fn is_scancode_down(&self, scancode: sdl2::scancode::ScanCode) -> bool {
        self.keyboard.contains(&scancode)
    }

    pub fn is_scancode_newly_down(&self, old: &SDLInput, scancode: sdl2::scancode::ScanCode) -> bool {
        !old.is_scancode_down(scancode) && self.is_scancode_down(scancode)
    }
}

fn solve_input(old: &SDLInput, new: &SDLInput, viewport: (i32, i32)) -> GameInput {
    use sdl2::mouse::Mouse;
    use sdl2::scancode::ScanCode;

    /// Screen coordinates (pixels) to normalized device coordinates (0..1)
    fn screen_to_ndc(viewport: (i32, i32), screen: (i32, i32)) -> (f32, f32) {
        let (width, height) = viewport;
        let (x, y) = screen;
        ((x as f32 / width as f32 - 0.5)*2.0, -(y as f32 / height as f32 - 0.5)*2.0)
    }

    fn screen_delta_to_y_ratio(viewport: (i32, i32), screen_delta: (i32, i32)) -> (f32, f32) {
        let (width, height) = viewport;
        let (x, y) = screen_delta;
        let x_aspect = (width as f32) / (height as f32);
        ((x as f32 / width as f32)*x_aspect, -(y as f32 / height as f32))
    }

    let hurl_all = new.is_scancode_newly_down(old, ScanCode::Space);
    let explode_subcube = new.is_mouse_button_down(Mouse::Left);
    let rearrange = new.is_mouse_button_newly_down(old, Mouse::Right);
    let reset = new.is_scancode_newly_down(old, ScanCode::R);
    let toggle_show_outlines = new.is_scancode_newly_down(old, ScanCode::O);
    let screen_pointer = match new.mouse_in_focus {
        true => match new.mouse {
            Some((_, x, y)) => Some((x, y)),
            None => None
        },
        false => None
    };

    let pointer = match screen_pointer {
        Some(screen) => Some(screen_to_ndc(viewport, screen)),
        None => None
    };

    let rotate_view = match new.get_mouse_delta(old, Mouse::Middle) {
        Some(d) => screen_delta_to_y_ratio(viewport, d),
        None => (0.0, 0.0)
    };

    let zoom_view_change = match (old.mouse_wheel_absolute, new.mouse_wheel_absolute) {
        ((_, old_y), (_, new_y)) => (new_y - old_y) as f32 / 3.0
    };

    GameInput {
        hurl_all: hurl_all,
        explode_subcube: explode_subcube,
        rearrange: rearrange,
        reset: reset,
        toggle_show_outlines: toggle_show_outlines,
        pointer: pointer,
        rotate_view: rotate_view,
        zoom_view_change: zoom_view_change
    }
}

impl Game {
    pub fn new(width: u16, height: u16) -> Result<Game, String> {
        use sdl2::video::{GLProfile, gl_attr};

        let sdl = try!(sdl2::init().video().build());

        gl_attr::set_context_profile(GLProfile::Core);
        gl_attr::set_context_version(3, 0);
        gl_attr::set_depth_size(24);
        gl_attr::set_double_buffer(true);

        let window = match sdl.window("Rust cubes demo", width as u32, height as u32).position_centered().opengl().resizable().build() {
            Ok(window) => window,
            Err(err) => return Err(format!("failed to create window: {}", err))
        };

        let context = match window.gl_create_context() {
            Ok(context) => context,
            Err(err) => return Err(format!("failed to create context: {}", err))
        };

        // Initialize the OpenGL function pointers
        gl::load_with(sdl2::video::gl_get_proc_address);

        let renderer = try!(Renderer::new());
        let state = GameState::new();

        Ok(Game {
            sdl: sdl,
            window: window,
            _context: context,
            renderer: renderer,
            state: state,
            mouse_wheel_absolute: (0, 0)
        })
    }

    fn frame_limit(&self) -> Option<u32> {
        // Twice the rate of a typical computer monitor
        // Some(120)
        None
    }

    fn event_loop(&mut self) -> SDLEventLoopResult {
        use sdl2::event::Event;
        use sdl2::keycode::KeyCode;

        let mut events = self.sdl.event_pump();

        for event in events.poll_iter() {
            match event {
                Event::Quit{..} => { return SDLEventLoopResult::Exit; },
                Event::KeyDown { keycode: key, .. } => {
                    if key == KeyCode::Escape {
                        return SDLEventLoopResult::Exit;
                    }
                },
                Event::MouseWheel { x, y, .. } => {
                    let (abs_x, abs_y) = self.mouse_wheel_absolute;
                    self.mouse_wheel_absolute = (abs_x + x, abs_y + y);
                },
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
        for (scancode, pressed) in keys {
            if pressed {
                keyboard.insert(scancode);
            }
        }

        SDLEventLoopResult::HasInput(SDLInput {
            keyboard: keyboard,
            mouse: Some(mouse),
            mouse_in_focus: mouse_in_focus,
            mouse_wheel_absolute: self.mouse_wheel_absolute
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
                SDLEventLoopResult::HasInput(input) => input,
                SDLEventLoopResult::Exit => break 'main
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
                    let ms = 1000/fps as u32 - (d*1000.0) as u32;
                    if ms > 0 {
                        sdl2::timer::delay(ms)
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
        self.window.properties_getters().get_size()
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
