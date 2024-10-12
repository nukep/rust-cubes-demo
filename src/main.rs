use std::mem::swap;

use cgmath::Matrix4;
use miniquad::*;

use glam::{vec3, Mat4, Vec3};

pub mod game;
pub mod util;

use game::{GameState, GameStepResult, GameInput};

// How much dragging the view affects rotation
#[cfg(target_arch = "wasm32")]
static DRAG_COEFF: f32 = 2.0;
#[cfg(not(target_arch = "wasm32"))]
static DRAG_COEFF: f32 = 8.0;

// How much scrolling the view affects zoom
#[cfg(target_arch = "wasm32")]
static ZOOM_COEFF: f32 = 1.0/100.0;
#[cfg(not(target_arch = "wasm32"))]
static ZOOM_COEFF: f32 = 0.5;

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,

    input: GameInput,
    drag_last: Option<(f32, f32)>,
    game_state: GameState,
    game_step_result: Option<GameStepResult>
}

impl Stage {
    fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        let cube_arrays = CubeArrays::new();

        let geometry_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&cube_arrays.vert_pos),
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&cube_arrays.indices),
        );

        let positions_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Vec3>(512),
        );

        let bindings = Bindings {
            vertex_buffers: vec![geometry_vertex_buffer, positions_vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };

        let shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shader::VERTEX,
                fragment: shader::FRAGMENT,
            }, 
            shader::meta()
        ).unwrap();

        let pipeline = ctx.new_pipeline(
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                VertexAttribute::with_buffer("position", VertexFormat::Float3, 0),
                VertexAttribute::with_buffer("in_inst_pos", VertexFormat::Float3, 1),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            }
        );

        Stage {
            ctx,
            pipeline,
            bindings,
            input: GameInput::new(),
            drag_last: None,
            game_state: GameState::new(),
            game_step_result: None
        }
    }

    // Change to -1 to 1 coordinates, where 0 is the center
    fn window_to_ndc_coordinates(x: f32, y: f32) -> (f32, f32) {
        let (width, height) = window::screen_size();
        let x = (x/width)*2.0 - 1.0;
        let y = -((y/height)*2.0 - 1.0);
        return (x, y)
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        let (width, height) = window::screen_size();
        let result = self.game_state.step((width as i32, height as i32), &self.input);
        self.game_step_result = Some(result);

        self.input.rearrange = false;
        self.input.hurl_all = false;
        self.input.reset = false;
        self.input.toggle_show_outlines = false;
        self.input.zoom_view_change = 0.0;
    }
    fn draw(&mut self) {
        let Some(result) = std::mem::replace(&mut self.game_step_result, None) else {
            return;
        };

        self.ctx.buffer_update(
            self.bindings.vertex_buffers[1],
            BufferSource::slice(&[0.0, 0.0, 0.0]),
        );

        let projection_view = cgmath_to_glam(result.projection_view);
        let show_outlines = if self.game_state.show_outlines { 1 } else { 0 };

        self.ctx.begin_default_pass(Default::default());
        self.ctx.clear(Some((0., 0., 0.5, 1.)), None, None);
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);

        let mut idx = 0;
        for subcube in &self.game_state.cube.subcubes {
            let l = 0.5 - subcube.subcube_length / 2.0;
            let pos = match subcube.segment {
                v => (v.x + l, v.y + l, v.z + l)
            };
            let pos = glam::Vec3::new(pos.0, pos.1, pos.2);

            let model = cgmath_to_glam(subcube.get_model_matrix());

            let hovered = match result.selected_subcube {
                Some(selected_idx) => idx == selected_idx,
                None => false
            };
            let hovered = if hovered { 1 } else { 0 };

            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
                projection_view: projection_view,
                model: model,
                show_outlines: show_outlines,
                hovered: hovered,
                cube_pos: pos,
                cube_size: subcube.subcube_length
            }));
            self.ctx.draw(0, 36, 1);

            idx += 1;
        }

        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let (x, y) = Stage::window_to_ndc_coordinates(x, y);
        self.input.pointer = Some((x, y));

        if let Some((lastx, lasty)) = self.drag_last {
            self.input.rotate_view = ((x - lastx) * DRAG_COEFF, (y - lasty) * DRAG_COEFF);
            self.drag_last = Some((x, y));
        } else {
            self.input.rotate_view = (0.0, 0.0)
        }
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.input.explode_subcube = true;
        }
        if button == MouseButton::Right {
            self.input.rearrange = true;
        }
        if button == MouseButton::Middle {
            self.input.rotate_view = (0.0, 0.0);
            self.drag_last = Some(Stage::window_to_ndc_coordinates(x, y));
        }
    }
    fn mouse_button_up_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        if button == MouseButton::Left {
            self.input.explode_subcube = false;
        }
        if button == MouseButton::Middle {
            self.drag_last = None;
        }
    }
    fn char_event(&mut self, c: char, _keymods: KeyMods, _repeat: bool) {
        let c = c.to_ascii_lowercase();
        if c == ' ' {
            self.input.hurl_all = true;
        }
        if c == 'r' {
            self.input.reset = true;
        }
        if c == 'o' {
            self.input.toggle_show_outlines = true;
        }
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        // +Y zooms in, -Y zooms out
        self.input.zoom_view_change = (y as f32) * ZOOM_COEFF;
    }
}

fn cgmath_to_glam(mat: cgmath::Matrix4<f32>) -> Mat4 {
    use cgmath::Matrix;
    let ptr = mat.as_ptr();
    let arr: &[f32; 16] = unsafe { std::mem::transmute(ptr) };
    Mat4::from_cols_array(arr)
}

fn main() {
    let conf = conf::Conf {
        window_title: "Rust Cubes Demo".to_string(),
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    };
    miniquad::start(conf, move || Box::new(Stage::new()));
}

struct CubeArrays {
    pub vert_pos: [f32; 6*4 * 3],
    pub indices: [u8; 6*6]
}

impl CubeArrays {
    pub fn new() -> CubeArrays{

        let offset = -0.5;
        let length = 1.0;
    
        // 8 corners in a cube
        let corner: [(f32,f32,f32); 8] = {
            let l = offset;
            let m = length + offset;
    
            [
                (m, m, m),
                (m, m, l),
                (m, l, m),
                (m, l, l),
                (l, m, m),
                (l, m, l),
                (l, l, m),
                (l, l, l),
            ]
        };
    
        // Which corners to copy to the vertex buffer for each face.
        // In order to maintain distinct normals for each face,
        // corners cannot be shared among different faces.
        static VERT_IDX: [usize; 4*6] = [
            0,1,2,3,
            4,0,6,2,
            5,4,7,6,
            1,5,3,7,
            5,1,4,0,
            3,7,2,6
        ];
    
        // Which vertices to form triangle faces from
        static IDX: [u8; 6*6] = [
            0,1,2, 1,3,2,
            4,5,6, 5,7,6,
            8,9,10, 9,11,10,
            12,13,14, 13,15,14,
            16,17,18, 17,19,18,
            20,21,22, 21,23,22
        ];
    
        let buffer: Vec<f32> = VERT_IDX.iter().flat_map(|&i| {
            let (x,y,z) = corner[i];
            vec![x, y, z].into_iter()
        }).collect();

        CubeArrays {
            vert_pos: buffer.try_into().unwrap(),
            indices: IDX
        }
    }
}


mod shader {
    use miniquad::*;

    pub const VERTEX: &str = include_str!("shaders/vertex.glsl");
    pub const FRAGMENT: &str = include_str!("shaders/fragment.glsl");

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("projection_view", UniformType::Mat4),
                    UniformDesc::new("model", UniformType::Mat4),
                    UniformDesc::new("show_outlines", UniformType::Int1),
                    UniformDesc::new("hovered", UniformType::Int1),
                    UniformDesc::new("cube_pos", UniformType::Float3),
                    UniformDesc::new("cube_size", UniformType::Float1),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub projection_view: glam::Mat4,
        pub model: glam::Mat4,
        pub show_outlines: u32,
        pub hovered: u32,
        pub cube_pos: glam::Vec3,
        pub cube_size: f32
    }

}