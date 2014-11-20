#![feature(macro_rules)]

extern crate cgmath;
extern crate util;

use cube::Cube;
pub mod cube;

/// GameState describes all non-derivable data required to present a frame.
/// It is perpetually updated and controlled by the game loop.
pub struct GameState {
    pub rot: f32,
    pub cube: cube::Cube,
    pub show_outlines: bool
}

/// One-off data derived from GameState and used by the renderer.
/// Unlike GameState, this data is never updated and is discarded after
/// use by the renderer.
pub struct GameStepResult {
    pub projection_view: cgmath::Matrix4<f32>,
    pub selected_subcube: Option<uint>
}

#[deriving(Default)]
pub struct GameInput {
    pub explode: bool,
    pub explode_subcube: bool,
    pub reset: bool,
    pub toggle_show_outlines: bool,
    /// The pointer coordinates range from -1.0 to +1.0.
    /// e.g. (0.0, 0.0) is the center, (1.0, 1.0) is the top-right.
    pub pointer: Option<(f32, f32)>,
    pub rotate_view: (f32, f32)
}

impl GameInput {
    pub fn new() -> GameInput { std::default::Default::default() }
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            rot: 0.0,
            cube: Cube::new(),
            show_outlines: false
        }
    }

    pub fn steps_per_second() -> int { 60 }

    pub fn step(&mut self, viewport: (i32,i32), input: &GameInput) -> GameStepResult {
        if input.explode {
            self.cube.try_explode(4.0);
        } else if input.reset {
            self.cube.try_reset();
        }

        let projection_view = self.solve_projection_view(viewport);

        let selected_subcube = self.solve_selected_subcube(projection_view, input.pointer);

        if input.explode_subcube {
            match selected_subcube {
                Some(s) => self.cube.explode_subcube_if_at_least(s, 4.0, 2, 1.0/16.0),
                None => ()
            }
        }

        if input.toggle_show_outlines {
            self.show_outlines = !self.show_outlines;
        }

        self.rot += 1.0;
        self.cube.step(1.0 / GameState::steps_per_second() as f32);

        GameStepResult {
            projection_view: projection_view,
            selected_subcube: selected_subcube
        }
    }

    fn solve_selected_subcube(&self, projection_view: cgmath::Matrix4<f32>, pointer: Option<(f32, f32)>) -> Option<uint> {
        use cgmath::{Matrix, Vector, Vector4, Point, Point3, EuclideanVector, Ray, Ray3};

        let mouse_ray: Option<Ray3<f32>> = match pointer {
            Some((x, y)) => {
                // From NDC to world coordinates
                let post_project_v1 = Vector4::new(x, y, -1.0, 1.0);
                let post_project_v2 = Vector4::new(x, y, 1.0, 1.0);

                let inv_projection_view = projection_view.invert().unwrap();
                let pre_project_p1 = Point3::from_homogeneous(&inv_projection_view.mul_v(&post_project_v1));
                let pre_project_p2 = Point3::from_homogeneous(&inv_projection_view.mul_v(&post_project_v2));

                let direction = pre_project_p2.sub_p(&pre_project_p1).normalize();

                Some(Ray::new(pre_project_p1, direction))
            },
            None => None
        };

        match mouse_ray {
            Some(ref ray) => {
                match self.cube.get_subcube_from_ray(ray) {
                    Some((index, _)) => Some(index),
                    None => None
                }
            },
            None => None
        }
    }

    fn solve_projection_view(&self, viewport: (i32,i32)) -> cgmath::Matrix4<f32> {
        use util::matrix::MatrixBuilder;
        use std::num::Float;

        let viewport_aspect = match viewport {
            (width, height) => width as f32 / height as f32
        };
        let projection = cgmath::ToMatrix4::to_matrix4(&cgmath::PerspectiveFov {
            fovy: cgmath::Deg { s: 45.0 },
            aspect: viewport_aspect,
            near: 1.0,
            far: 100.0
        });

        let rad = 20.0 * Float::pi()/180.0;
        let view = cgmath::Matrix4::identity()
            .translate(0.0, 0.0, -5.0)
            .rotate_y(self.rot * Float::pi()/180.0)
            .rotate_x(rad);

        projection * view
    }
}
