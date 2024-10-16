use std;

use cgmath::prelude::*;
use cgmath::{Vector3, Quaternion};
use collision::{Ray3};

struct CubeStateRearranging {
    p: f32,
    next_state: Box<CubeState>
}

enum CubeState {
    Simulating,
    Resetting,
    Rearranging(CubeStateRearranging)
}

pub struct Cube {
    pub subcubes: Vec<Subcube>,
    state: CubeState
}

#[derive(Copy, Clone)]
pub struct Subcube {
    pub segment: Vector3<f32>,
    pub subcube_length: f32,
    pub pos: Vector3<f32>,
    pub orientation: Quaternion<f32>,

    vel: Vector3<f32>,
    angular_momentum: Vector3<f32>
}

impl Cube {
    pub fn new() -> Cube {
        let subcubes = vec![Subcube::from_segment(Zero::zero(), 1.0)];

        Cube {
            subcubes: subcubes,
            state: CubeState::Simulating
        }
    }

    /// Run the provided callback if the Simulating state is active.
    ///
    /// Note: A reference to self is passed to the callback to get around
    /// Rust's borrowing rules.
    fn try_on_simulating<F, R>(&mut self, cb: F) -> Option<R> where
        F: FnOnce(&mut Cube) -> R
    {
        match self.state {
            CubeState::Simulating => {
                Some(cb(self))
            },
            _ => None
        }
    }

    pub fn try_hurl_all(&mut self, force: f32) {
        self.try_on_simulating(|_self| {
            let origin = Vector3::from_value(0.0);
            for subcube in _self.subcubes.iter_mut() {
                subcube.hurl(force, &origin);
            }
        });
    }

    pub fn try_rearrange(&mut self) {
        self.try_on_simulating(|_self| {
            for subcube in _self.subcubes.iter_mut() {
                subcube.cancel_momentum();
            }
            _self.state = CubeState::Rearranging(CubeStateRearranging{
                p: 0.0,
                next_state: Box::new(CubeState::Simulating)
            })
        });
    }

    pub fn try_reset(&mut self) {
        self.try_on_simulating(|_self| {
            for subcube in _self.subcubes.iter_mut() {
                subcube.cancel_momentum();
            }
            _self.state = CubeState::Rearranging(CubeStateRearranging{
                p: 0.0,
                next_state: Box::new(CubeState::Resetting)
            })
        });
    }

    fn subdivide_subcube(&mut self, index: usize, subdivide_count: u32) -> Vec<usize> {
        assert!(subdivide_count > 0);
        let original = self.subcubes[index];

        // Alter the subcube at the specified index
        self.subcubes[index] = original.get_subdivided_subcube(subdivide_count, (0,0,0));

        // Push `subdivide_count**3 - 1` new subcubes at the end of the `subcubes` vector
        let subdivide_count_cubed = subdivide_count.pow(3);
        self.subcubes.reserve(subdivide_count_cubed as usize - 1);

        let new_subcubes_idx = self.subcubes.len();

        self.subcubes.extend((1..subdivide_count_cubed).map(|i| {
            let x = i%subdivide_count;
            let y = (i/subdivide_count)%(subdivide_count);
            let z = i/subdivide_count/subdivide_count;

            original.get_subdivided_subcube(subdivide_count, (x,y,z))
        }));

        let mut result = Vec::with_capacity(subdivide_count_cubed as usize);
        result.push(index);
        result.extend(new_subcubes_idx..self.subcubes.len());

        result
    }

    pub fn explode_subcube(&mut self, index: usize, force: f32, subdivide_count: u32) {
        let origin = self.subcubes[index].pos;

        let subcubes_idx = self.subdivide_subcube(index, subdivide_count);
        for &subcube_idx in subcubes_idx.iter() {
            let subcube = &mut self.subcubes[subcube_idx];
            subcube.hurl(force, &origin);
        }
    }

    pub fn explode_subcube_if_at_least(&mut self, index: usize, force: f32, subdivide_count: u32, min_subcube_length: f32) {
        if self.subcubes[index].subcube_length >= min_subcube_length {
            self.explode_subcube(index, force, subdivide_count);
        } else {
            // Still hurl the subcube
            let s = &mut self.subcubes[index];
            let origin = s.pos;
            s.hurl(force, &origin);
        }
    }

    /// Integrate the cube simulation by stepping all subcubes
    pub fn step(&mut self, frac: f32) {
        let next_state = match self.state {
            CubeState::Simulating => {
                for subcube in self.subcubes.iter_mut() {
                    subcube.step(frac);
                }
                None
            },
            CubeState::Resetting => {
                self.subcubes = vec![Subcube::from_segment(Zero::zero(), 1.0)];

                Some(CubeState::Simulating)
            },
            CubeState::Rearranging(ref mut s) => {
                for subcube in self.subcubes.iter_mut() {
                    subcube.approach_original_arrangement(frac);
                }

                // Go to the next state after 1.5 seconds
                s.p += frac;
                match s.p {
                    0.0..=1.5 => None,
                    _ => {
                        for subcube in self.subcubes.iter_mut() {
                            subcube.reset();
                        }

                        // Use a dummy value to swap in the next state
                        let mut next_state = CubeState::Simulating;
                        // Moving would violate lifetime rules, so a swap is
                        // used instead.
                        std::mem::swap(&mut next_state, &mut *s.next_state);

                        Some(next_state)
                    }
                }
            }
        };

        match next_state {
            Some(s) => self.state = s,
            None => ()
        };
    }

    /// Get the closest subcube that intersects with the ray.
    /// Returns a Some tuple with the index and a reference to the subcube
    /// if one intersects with the ray.
    /// Returns None if no subcube intersects with the ray.
    pub fn get_subcube_from_ray(&self, ray: &Ray3<f32>) -> Option<(usize, &Subcube)> {
        use collision::Ray;
        use crate::util::compare::CompareSmallest;
        use std::cmp::Ordering;

        fn intersects_with_unit_cube(ray: &Ray3<f32>) -> Option<f32> {
            use cgmath::Point3;
            use collision::{Continuous, Plane};
            // The unit cube is at the origin, from -0.5..+0.5

            static PLANES: [Plane<f32>; 6] = [
                Plane { n: Vector3::new( 1.0,  0.0,  0.0), d: 0.5 },
                Plane { n: Vector3::new(-1.0,  0.0,  0.0), d: 0.5 },
                Plane { n: Vector3::new( 0.0,  1.0,  0.0), d: 0.5 },
                Plane { n: Vector3::new( 0.0, -1.0,  0.0), d: 0.5 },
                Plane { n: Vector3::new( 0.0,  0.0,  1.0), d: 0.5 },
                Plane { n: Vector3::new( 0.0,  0.0, -1.0), d: 0.5 },
            ];

            let mut closest: Option<f32> = None;

            for plane in PLANES.iter() {
                if let Some(point) = plane.intersection(ray) {
                    let Point3{x, y, z} = point;

                    match (x, y, z) {
                        // Intersected point must be within bounds
                        (-0.5..=0.5, -0.5..=0.5, -0.5..=0.5) => {
                            let diff = point - ray.origin;
                            closest.set_if_smallest(diff.magnitude());
                        },
                        _ => ()
                    }
                }
            }

            closest
        }

        struct SubcubeDistance<'a>(usize, &'a Subcube, f32);

        impl<'a> PartialEq for SubcubeDistance<'a> {
            fn eq(&self, other: &SubcubeDistance) -> bool {
                let (self_dist, other_dist) = (self.2, other.2);
                self_dist.eq(&other_dist)
            }
        }

        impl<'a> PartialOrd for SubcubeDistance<'a> {
            fn partial_cmp(&self, other: &SubcubeDistance) -> Option<Ordering> {
                let (self_dist, other_dist) = (self.2, other.2);
                self_dist.partial_cmp(&other_dist)
            }
        }

        // Option tuple of: index, subcube, distance
        let mut closest_subcube: Option<SubcubeDistance> = None;

        for (index, subcube) in self.subcubes.iter().enumerate() {
            // Transform ray relative to a non-rotated unit cube
            let new_ray = {
                let q = subcube.orientation.invert();
                // Make ray relative to center of subcube
                let origin = (ray.origin - subcube.pos) / subcube.subcube_length;

                // Rotate ray around center of subcube
                Ray::new(q.rotate_point(origin), q.rotate_vector(ray.direction))
            };

            if let Some(dist) = intersects_with_unit_cube(&new_ray) {
                assert!(dist >= 0.0);
                closest_subcube.set_if_smallest(SubcubeDistance(index, subcube, dist*subcube.subcube_length));
            }
        }

        match closest_subcube {
            Some(SubcubeDistance(idx, subcube, _)) => Some((idx, subcube)),
            None => None
        }
    }
}

impl Subcube {
    fn from_segment(segment: Vector3<f32>, subcube_length: f32) -> Subcube {
        Subcube {
            segment: segment,
            subcube_length: subcube_length,
            pos: segment,
            vel: Zero::zero(),
            orientation: Quaternion::one(),
            angular_momentum: Zero::zero()
        }
    }

    pub fn get_model_matrix(&self) -> cgmath::Matrix4<f32> {
        use crate::util::matrix::MatrixBuilder;
        cgmath::Matrix4::identity()
            .translate_v(&self.pos)
            .scale_s(self.subcube_length)
            .quaternion(&self.orientation)
    }

    fn get_subdivided_subcube(&self, subdivide_count: u32, loc: (u32, u32, u32)) -> Subcube {
        use cgmath::Matrix4;
        use crate::util::matrix::MatrixBuilder;

        /// Vector is relative to corner of subcube, bounded 0..1
        /// i.e. location of (0,0,0) will return a Vector of (0,0,0)
        fn new_pos(subdivide_count: u32, loc: (u32, u32, u32)) -> Vector3<f32> {
            let (x,y,z) = loc;
            (Vector3::new(x as f32, y as f32, z as f32) / (subdivide_count as f32)) + Vector3::from_value((1.0 / subdivide_count as f32) / 2.0)
        }

        fn matrix_mul_v3(mtx: &Matrix4<f32>, v: &Vector3<f32>) -> Vector3<f32> {
            (mtx * v.extend(1.0)).truncate()
        }

        // Transform subcube from a no-rotation, unit cube with its origin at the lower-front-left corner
        let segment_model = Matrix4::identity()
            .translate_v(&self.segment)
            .scale_s(self.subcube_length)
            .translate_s(-0.5);

        let model = self.get_model_matrix()
            .translate_s(-0.5);

        let interpolated_pos = new_pos(subdivide_count, loc);

        Subcube {
            segment: matrix_mul_v3(&segment_model, &interpolated_pos),
            subcube_length: self.subcube_length / subdivide_count as f32,
            pos: matrix_mul_v3(&model, &interpolated_pos),
            vel: self.vel,
            orientation: self.orientation,
            angular_momentum: self.angular_momentum,
        }
    }

    /// Add velocity and angular momentum to the subcube.
    ///
    /// The subcube will tend to repel from the specified origin.
    /// Some psudo-random variance will also be randomly added to the velocity and angular momentum.
    pub fn hurl(&mut self, force: f32, origin: &Vector3<f32>) {
        fn random_vector3() -> Vector3<f32> {
            Vector3::new(quad_rand::gen_range(-1.0, 1.0), quad_rand::gen_range(-1.0, 1.0), quad_rand::gen_range(-1.0, 1.0))
        }

        let v = (self.pos - origin) * (16.0);
        self.vel = (v + random_vector3() * (4.0)) * (force*0.1);
        self.angular_momentum = (v + random_vector3() * (0.5)) * (force*0.5);
    }

    fn reset(&mut self) {
        *self = Subcube::from_segment(self.segment, self.subcube_length);
    }

    fn cancel_momentum(&mut self) {
        self.vel = Zero::zero();
        self.angular_momentum = Zero::zero();
    }

    fn approach_original_arrangement(&mut self, _frac: f32) {
        let target_subcube = Subcube::from_segment(self.segment, self.subcube_length);

        // TODO - figure out the math on this (using frac)
        let lerp_amount = 0.1;

        self.pos = self.pos.lerp(target_subcube.pos, lerp_amount);
        self.orientation = self.orientation.nlerp(target_subcube.orientation, lerp_amount);
    }

    fn step(&mut self, frac: f32) {
        use std::f32;
        
        // **Velocity** //
        self.pos += self.vel * frac;

        // **Angular momentum** //
        let q_angular_momentum = Quaternion::from_sv(0.0, self.angular_momentum * frac);

        // Derivative of orientation
        let d_orientation = q_angular_momentum * self.orientation;
        self.orientation = (self.orientation + d_orientation).normalize();

        // Slow down 30% per second
        // x^(1/frac) = 0.7
        // x = 0.7 ^ frac
        let m = f32::powf(0.7, frac);
        self.vel *= m;
        self.angular_momentum *= m;
    }
}
