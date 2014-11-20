use gl;
use game::{GameState, GameStepResult};

use opengl_util;
use opengl_util::vertex::VertexArray;

pub struct Renderer {
    program: opengl_util::shader::Program,
    vao: VertexArray
}

fn load_default_program() -> opengl_util::shader::Program {
    use opengl_util::shader::{Shader, Program};

    let vertex_source = include_str!("shaders/vertex.glsl");
    let fragment_source = include_str!("shaders/fragment.glsl");

    let vertex = match Shader::vertex_from_source(vertex_source) {
        Ok(shader) => shader,
        Err(s) => panic!("Vertex shader compilation error: {}", s)
    };
    let fragment = match Shader::fragment_from_source(fragment_source) {
        Ok(shader) => shader,
        Err(s) => panic!("Fragment shader compilation error: {}", s)
    };

    match Program::link("default".to_string(), [&vertex, &fragment]) {
        Ok(program) => program,
        Err(s) => panic!("Shader link error: {}", s)
    }
}

impl Renderer {
    pub fn new() -> Result<Renderer, String> {
        let program = load_default_program();

        let a_position = program.get_attrib("position");
        let vao = opengl_util::shape::gen_cube(1.0, -0.5, a_position);

        Ok(Renderer {
            program: program,
            vao: vao
        })
    }

    pub fn render(&mut self, state: &GameState, step_result: &GameStepResult, viewport: (i32, i32)) {
        use cgmath::FixedArray;

        let (viewport_width, viewport_height) = viewport;

        unsafe {
            gl::Viewport(0, 0, viewport_width, viewport_height);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);
        };

        let u_show_outlines = self.program.get_uniform("show_outlines");
        let u_hovered = self.program.get_uniform("hovered");
        let u_model = self.program.get_uniform("model");
        let u_cube_pos = self.program.get_uniform("cube_pos");
        let u_cube_size = self.program.get_uniform("cube_size");

        self.program.use_program(|uniform| {
            uniform.set_mat4(self.program.get_uniform("projection_view"), step_result.projection_view.as_fixed());
            uniform.set_bool(u_show_outlines, state.show_outlines);

            self.vao.bind_vao(|vao_ctx| {
                let mut idx = 0;
                for subcube in state.cube.subcubes.iter() {
                    let l = 0.5 - subcube.subcube_length / 2.0;
                    let pos = match subcube.segment {
                        v => (v.x + l, v.y + l, v.z + l)
                    };

                    let model = subcube.get_model_matrix();

                    let hovered = match step_result.selected_subcube {
                        Some(selected_idx) => (idx == selected_idx),
                        None => false
                    };

                    uniform.set_bool(u_hovered, hovered);
                    uniform.set_mat4(u_model, model.as_fixed());
                    uniform.set_vec3(u_cube_pos, pos);
                    uniform.set_float(u_cube_size, subcube.subcube_length);

                    vao_ctx.draw_elements(gl::TRIANGLES, 6*6, 0);
                    idx += 1;
                }
            });
        });
    }
}
