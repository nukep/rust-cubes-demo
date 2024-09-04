use cgmath::{Matrix3, Matrix4, Vector3, Quaternion, Rad, SquareMatrix, BaseFloat};

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

impl<S: BaseFloat> MatrixBuilder<S, Vector3<S>, Quaternion<S>> for Matrix4<S> {
    fn scale(&self, x: S, y: S, z: S) -> Matrix4<S> {
        self.scale_v(&Vector3::new(x,y,z))
    }

    fn scale_v(&self, value: &Vector3<S>) -> Matrix4<S> {
        self * Matrix4::from(Matrix3::from_diagonal(*value))
    }

    fn scale_s(&self, value: S) -> Matrix4<S> {
        self * Matrix4::from(Matrix3::from_value(value))
    }

    fn rotate_x(&self, rad: S) -> Matrix4<S> {
        self * Matrix4::from_angle_x(Rad(rad))
    }

    fn rotate_y(&self, rad: S) -> Matrix4<S> {
        self * Matrix4::from_angle_y(Rad(rad))
    }

    fn rotate_z(&self, rad: S) -> Matrix4<S> {
        self * Matrix4::from_angle_z(Rad(rad))
    }

    fn quaternion(&self, value: &Quaternion<S>) -> Matrix4<S> {
        self * Matrix4::from(*value)
    }

    fn translate(&self, x: S, y: S, z: S) -> Matrix4<S> {
        self.translate_v(&Vector3::new(x,y,z))
    }

    fn translate_v(&self, disp: &Vector3<S>) -> Matrix4<S> {
        self * Matrix4::from_translation(*disp)
    }

}
