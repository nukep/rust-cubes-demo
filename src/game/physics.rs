///! Note that "Physics" is in massive quotation marks.
///! This does not aim to be a realistic MKS physics simulation.

use cgmath::{Quaternion, Point3, Vector3, BaseFloat, Point, Vector};
use std::num::Float;

fn integrate_decay<T: Float>(decay: T, time: T) -> T {
    // x^(1/time) = 1-decay
    // x = (1-decay)^time
    let one: T = Float::one();
    (one - decay).powf(time)
}

pub struct ScalarMotion<T> {
    pub scalar: T,
    pub change: T,
    /// How much `change` will be reduced by every second as a percentage.
    /// Ranges from 0 to 1. 0 is no decay, 1 is complete decay after 1 second.
    pub decay: T
}
impl<T: Float> ScalarMotion<T> {
    pub fn new(scalar: T, change: T, decay: T) -> ScalarMotion<T> {
        ScalarMotion {
            scalar: scalar,
            change: change,
            decay: decay
        }
    }
    pub fn step(&mut self, frac: T) {
        self.scalar = self.scalar + self.change * frac;
        self.change = self.change * integrate_decay(self.decay, frac);
    }
}

pub struct QuaternionMotion<T: BaseFloat> {
    pub quaternion: Quaternion<T>,
    pub angular_momentum: Vector3<T>,
    /// Ranges from 0 to 1. 0 is no decay, 1 is complete decay after 1 second.
    pub decay: T
}
impl<T: BaseFloat> QuaternionMotion<T> {
    pub fn new(quaternion: Quaternion<T>, angular_momentum: Vector3<T>, decay: T) -> QuaternionMotion<T> {
        QuaternionMotion {
            quaternion: quaternion,
            angular_momentum: angular_momentum,
            decay: decay
        }
    }
    pub fn step(&mut self, frac: T) {
        let q_angular_momentum = Quaternion::from_sv(Float::zero(), self.angular_momentum.mul_s(frac));
        let d_quaternion = q_angular_momentum.mul_q(&self.quaternion);
        self.quaternion = self.quaternion.add_q(&d_quaternion).normalize();
        self.angular_momentum = self.angular_momentum.mul_s(integrate_decay(self.decay, frac));
    }
}

pub struct PointMotion<T: BaseFloat> {
    pub point: Point3<T>,
    pub velocity: Vector3<T>,
    /// How much `velocity` will be reduced by every second as a percentage.
    /// Ranges from 0 to 1. 0 is no decay, 1 is complete decay after 1 second.
    pub decay: T
}
impl<T: BaseFloat> PointMotion<T> {
    pub fn step(&mut self, frac: T) {
        self.point.add_self_v(&self.velocity.mul_s(frac));
        self.velocity.mul_self_s(integrate_decay(self.decay, frac));
    }
}
