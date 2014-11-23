#![feature(macro_rules)]

extern crate cgmath;
extern crate util;

use cube::Cube;

pub mod cube;
mod physics;

/// GameState describes all non-derivable data required to present a frame.
/// It is perpetually updated and controlled by the game loop.
pub struct GameState {
    pub cube: cube::Cube,
    pub show_outlines: bool,
    orientation: physics::QuaternionMotion<f32>,
    zoom: physics::ScalarMotion<f32>
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
    pub hurl_all: bool,
    pub explode_subcube: bool,
    pub rearrange: bool,
    pub reset: bool,
    pub toggle_show_outlines: bool,
    /// The pointer coordinates range from -1.0 to +1.0.
    /// e.g. (0.0, 0.0) is the center, (1.0, 1.0) is the top-right.
    pub pointer: Option<(f32, f32)>,
    pub rotate_view: (f32, f32),
    pub zoom_view_change: f32
}

impl GameInput {
    pub fn new() -> GameInput { std::default::Default::default() }
}

impl GameState {
    pub fn new() -> GameState {
        use cgmath::{Vector3, Rotation};

        GameState {
            cube: Cube::new(),
            show_outlines: true,
            orientation: physics::QuaternionMotion::new(
                Rotation::look_at(&Vector3::new(0.5, 0.25, 0.5), &Vector3::new(0.0, 1.0, 0.0)),
                Vector3::new(0.0, 0.2, 0.0),
                0.5
            ),
            zoom: physics::ScalarMotion::new(0.5, 0.2, 0.9)
        }
    }

    pub fn steps_per_second() -> int { 60 }

    pub fn step(&mut self, viewport: (i32,i32), input: &GameInput) -> GameStepResult {
        let frac = 1.0 / GameState::steps_per_second() as f32;

        if input.hurl_all {
            self.cube.try_hurl_all(4.0);
        } else if input.rearrange {
            self.cube.try_rearrange();
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

        {
            let (x,y) = input.rotate_view;
            if (x,y) != (0.0,0.0) {
                use cgmath::{Vector, Vector3};
                let ang = Vector3::new(-y, x, 0.0).mul_s(32.0);
                self.orientation.angular_momentum = ang;
            }
        }
        self.zoom.change -= input.zoom_view_change * 1.0/2.0;

        self.orientation.step(frac);
        self.zoom.step(frac);
        self.cube.step(frac);

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
            near: 0.1,
            far: 100.0
        });

        let view = cgmath::Matrix4::identity()
            .translate(0.0, 0.0, -1.0 + -(5.0f32.powf(self.zoom.scalar)))
            .quaternion(&self.orientation.quaternion);

        projection * view
    }
}
