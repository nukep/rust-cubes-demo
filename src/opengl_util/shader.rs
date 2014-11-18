use std;
use gl;
use gl::types::{GLint, GLuint};
use super::vertex::Attrib;

/// Encapsulates an OpenGL shader.
///
/// The shader may be a vertex or fragment shader.
pub struct Shader {
    pub id: GLuint
}
impl Drop for Shader {
    /// Deletes the shader (`glDeleteShader()`)
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) };
    }
}
impl Shader {
    /// Compiles a vertex shader from source.
    pub fn vertex_from_source(source: &str) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    /// Compiles a fragment shader from source.
    pub fn fragment_from_source(source: &str) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }

    fn from_source(source: &str, shader_type: GLuint) -> Result<Shader, String> {
        let shader = Shader {
            id: unsafe { gl::CreateShader(shader_type) }
        };
        source.with_c_str(|c_str| {
            unsafe { gl::ShaderSource(shader.id, 1, &c_str, std::ptr::null()); }
        });

        let successful: bool;

        unsafe {
            gl::CompileShader(shader.id);

            successful = {
                let mut result: GLint = 0;
                gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut result);
                result != 0
            };
        }

        if successful {
            Ok(shader)
        } else {
            Err(shader.get_compilation_log())
        }
    }

    fn get_compilation_log(&self) -> String {
        let mut len = 0;
        unsafe { gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut len) };
        assert!(len > 0);

        let mut buf = Vec::from_elem(len as uint, 0u8);
        let buf_ptr = buf.as_mut_ptr() as *mut gl::types::GLchar;
        unsafe { gl::GetShaderInfoLog(self.id, len, std::ptr::null_mut(), buf_ptr) };

        String::from_utf8(buf).unwrap()
    }
}

/// Encapsulates an OpenGL shader program.
pub struct Program {
    pub name: String,
    pub id: GLuint
}
impl Drop for Program {
    /// Deletes the program (`glDeleteProgram()`)
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) };
    }
}
impl Program {
    /// Links a new program by using the provided vertex and fragment shaders.
    ///
    /// Uses `glAttachShader()` and `glLinkProgram()`
    pub fn new(name: String, vertex: Shader, fragment: Shader) -> Program {
        let program = Program {
            name: name,
            id: unsafe { gl::CreateProgram() }
        };

        let successful: bool;

        unsafe {
            gl::AttachShader(program.id, vertex.id);
            gl::AttachShader(program.id, fragment.id);
            gl::LinkProgram(program.id);

            successful = {
                let mut result: GLint = 0;
                gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut result);
                result != 0
            };
        }

        if successful {
            program
        } else {
            panic!("Shader link error: {}", program.get_link_log());
        }
    }

    /// Calls `glUseProgram()` and then calls the `cb` closure, which is
    /// sent a context for assigning program uniforms.
    pub fn use_program(&self, cb: |ProgramUniformContext|) {
        unsafe { gl::UseProgram(self.id) };
        cb(ProgramUniformContext);
    }

    pub fn get_attrib(&self, name: &str) -> Attrib {
        match name.with_c_str(|ptr| unsafe { gl::GetAttribLocation(self.id, ptr) }) {
            -1 => panic!("Could not find attribute \"{}\" in shader program \"{}\"", name, self.name),
            attr => Attrib { id: attr as GLuint }
        }
    }

    pub fn get_uniform(&self, name: &str) -> Uniform {
        match self.get_uniform_option(name) {
            Some(uniform) => uniform,
            None => panic!("Could not find uniform \"{}\" in shader program \"{}\"", name, self.name)
        }
    }

    pub fn get_uniform_option(&self, name: &str) -> Option<Uniform> {
        match name.with_c_str(|c_str| unsafe { gl::GetUniformLocation(self.id, c_str) }) {
            -1 => None,
            id => Some(Uniform { id: id })
        }
    }

    fn get_link_log(&self) -> String {
        let mut len = 0;
        unsafe { gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut len) };
        assert!(len > 0);

        let mut buf = Vec::from_elem(len as uint, 0u8);
        let buf_ptr = buf.as_mut_ptr() as *mut gl::types::GLchar;
        unsafe { gl::GetProgramInfoLog(self.id, len, std::ptr::null_mut(), buf_ptr) };

        String::from_utf8(buf).unwrap()
    }
}

/// ProgramUniformContext represents a valid context for assigning Program uniforms
///
/// The motivation for `ProgramUniformContext` is to limit access to Program
/// uniform setters for once a Program is in use.
/// The `glUniform...()` functions must be called after a `glUseProgram()`.
/// It's easy to invoke OpenGL functions in the wrong order, so this is a way
/// to enforce the correct order.
pub struct ProgramUniformContext;
impl ProgramUniformContext {
    /// Corresponds to `glUniform1i()`
    pub fn set_i32(&self, u: Uniform, v: i32) { unsafe { gl::Uniform1i(u.id, v); }; }
    /// Corresponds to `glUniform1f()`
    pub fn set_float(&self, u: Uniform, v: f32) { unsafe { gl::Uniform1f(u.id, v); }; }
    /// Corresponds with `glUniform1i()`
    pub fn set_bool(&self, u: Uniform, v: bool) {
        unsafe { gl::Uniform1i(u.id, match v { true => 1, false => 0 } ); };
    }

    /// Corresponds to `glUniform3f()`
    pub fn set_vec3(&self, u: Uniform, v: (f32,f32,f32)) {
        let (x,y,z) = v;
        unsafe { gl::Uniform3f(u.id, x,y,z); };
    }

    /// Corresponds to `glUniformMatrix4fv()`
    pub fn set_mat4(&self, u: Uniform, mat: &[[f32, ..4], ..4]) {
        unsafe {
            let ptr: *const f32 = std::mem::transmute(mat);
            gl::UniformMatrix4fv(u.id, 1, gl::FALSE, ptr);
        };
    }
}

/// Encapsulates an OpenGL program uniform.
#[deriving(Clone)]
pub struct Uniform {
    pub id: GLint
}

#[test]
fn uniform_mat4_cast() {
    // We're assuming that the contents of Matrix4 transmutes cleanly.
    // I'm mainly worried about changes to Matrix4, or if
    // padding between the Vector4 fields could be introduced.
    // In addition, who knows what Rust nightly will break...

    let m = Matrix4::<f32>::new(0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0);

    unsafe {
        let ptr: *const f32 = std::mem::transmute(&m);
        for i in range(0, 16) {
            let v: f32 = std::ptr::read(ptr.offset(i));
            assert_eq!(v, i as f32);
        }
    };
}
