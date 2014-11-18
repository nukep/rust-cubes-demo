use std;
use lodepng;
use gl;
use gl::types::{GLint, GLuint, GLsizei};

pub struct Texture2D {
    id: GLuint
}
impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) };
    }
}
impl Texture2D {
    pub fn bind(&self, unit: u32) {
        check_max_texture_image_units(unit);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

#[cfg(debug)]
fn check_max_texture_image_units(unit: u32) {
    let max_texture_image_units = unsafe {
        let mut i = 0;
        gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut i);
        i as u32
    };
    if unit >= max_texture_image_units {
        panic!("Unit \"{}\" exceeds max texture image units of \"{}\"", unit, max_texture_image_units);
    }
}

#[cfg(not(debug))]
fn check_max_texture_image_units(_unit: u32) {
    // Do nothing
}

fn load_png24_data_and_upload(png_data: &[u8]) -> Result<Texture2D, String> {
    let img = match lodepng::decode24(png_data) {
        Ok(img) => img,
        Err(e) => return Err(format!("LodePNG decoding error: {}", e))
    };

    let tex_id: GLuint = unsafe {
        let mut id = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);
        id
    };

    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    }

    unsafe {
        let ptr = std::mem::transmute(img.buffer.get(0).unwrap());
        let internal = gl::RGB8 as GLint;
        let format = gl::RGB;
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            internal,
            img.width as GLsizei,
            img.height as GLsizei,
            0,
            format,
            gl::UNSIGNED_BYTE,
            ptr
        );
    }

    Ok(Texture2D { id: tex_id })
}
