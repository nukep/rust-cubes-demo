///! Note that "Physics" is in massive quotation marks.
///! This does not aim to be a realistic MKS physics simulation.

use cgmath::{Quaternion, Vector3, BaseFloat, InnerSpace};
use num::traits::{Float, Zero, One};

fn integrate_decay<T: Float + One>(decay: T, time: T) -> T {
    // x^(1/time) = 1-decay
    // x = (1-decay)^time
    let one: T = One::one();
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
        let q_angular_momentum = Quaternion::from_sv(Zero::zero(), self.angular_momentum * frac);
        let d_quaternion = q_angular_momentum * self.quaternion;
        self.quaternion = (self.quaternion + d_quaternion).normalize();
        self.angular_momentum = self.angular_momentum * integrate_decay(self.decay, frac);
    }
}
