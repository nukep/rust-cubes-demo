use cgmath::{Matrix, Matrix3, Matrix4, ToMatrix4, Vector3, Quaternion, Rad};

pub trait MatrixBuilder<S: Copy, V, Q>: Sized {
    fn scale(&self, x: S, y: S, z: S) -> Self;
    fn scale_v(&self, value: &V) -> Self;
    fn rotate_x(&self, rad: S) -> Self;
    fn rotate_y(&self, rad: S) -> Self;
    fn rotate_z(&self, rad: S) -> Self;
    fn quaternion(&self, value: &Q) -> Self;
    fn translate(&self, x: S, y: S, z: S) -> Self;
    fn translate_v(&self, disp: &V) -> Self;

    fn scale_s(&self, value: S) -> Self {
        self.scale(value, value, value)
    }
    fn translate_s(&self, value: S) -> Self {
        self.translate(value, value, value)
    }
}

// I can't figure out generics right now
// I'm always getting cryptic "the parameter type `S` may not live long enough" errors
// Why, though?! BaseFloat should satisfy every trait that's used by Vector3...
type S = f32;

// impl<S: BaseFloat> MatrixBuilder<S, Vector3<S>> for Matrix4<S>
impl MatrixBuilder<S, Vector3<S>, Quaternion<S>> for Matrix4<S> {
    fn scale(&self, x: S, y: S, z: S) -> Matrix4<S> {
        self.scale_v(&Vector3::new(x,y,z))
    }

    fn scale_v(&self, value: &Vector3<S>) -> Matrix4<S> {
        self.mul_m(&Matrix3::from_diagonal(value).to_matrix4())
    }

    fn scale_s(&self, value: S) -> Matrix4<S> {
        self.mul_m(&Matrix3::from_value(value).to_matrix4())
    }

    fn rotate_x(&self, rad: S) -> Matrix4<S> {
        self.mul_m(&Matrix3::from_angle_x(Rad { s: rad }).to_matrix4())
    }

    fn rotate_y(&self, rad: S) -> Matrix4<S> {
        self.mul_m(&Matrix3::from_angle_y(Rad { s: rad }).to_matrix4())
    }

    fn rotate_z(&self, rad: S) -> Matrix4<S> {
        self.mul_m(&Matrix3::from_angle_z(Rad { s: rad }).to_matrix4())
    }

    fn quaternion(&self, value: &Quaternion<S>) -> Matrix4<S> {
        self.mul_m(&value.to_matrix4())
    }

    fn translate(&self, x: S, y: S, z: S) -> Matrix4<S> {
        self.translate_v(&Vector3::new(x,y,z))
    }

    fn translate_v(&self, disp: &Vector3<S>) -> Matrix4<S> {
        self.mul_m(&Matrix4::from_translation(disp))
    }

}
