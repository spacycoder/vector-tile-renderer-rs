use gl;
use std::{ffi::CString, path::Path, ptr, str};

#[derive(Clone, Copy)]
pub struct Shader {
    pub program_id: u32,
}

impl Shader {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Self {
        unsafe {
            let mut shader_builder = ShaderBuilder::new();
            shader_builder = shader_builder.attach_file(vertex_shader_path);
            shader_builder = shader_builder.attach_file(fragment_shader_path);
            shader_builder.link()
        }
    }

    // Make sure the shader is active before calling this
    pub unsafe fn get_uniform_location(&self, name: &str) -> i32 {
        gl::GetUniformLocation(
            self.program_id,
            CString::new(name).expect("CString::new failed").as_ptr(),
        )
    }

    pub unsafe fn activate(&self) {
        gl::UseProgram(self.program_id);
    }

    pub unsafe fn set_int(&self, name: &str, value: i32) {
        gl::Uniform1i(self.get_uniform_location(name), value);
    }

    pub unsafe fn set_float(&self, name: &str, value: f32) {
        gl::Uniform1f(self.get_uniform_location(name), value);
    }

    pub unsafe fn set_vec2(&self, name: &str, x: f32, y: f32) {
        gl::Uniform2f(self.get_uniform_location(name), x, y);
    }

    pub unsafe fn set_vec3(&self, name: &str, x: f32, y: f32, z: f32) {
        gl::Uniform3f(self.get_uniform_location(name), x, y, z);
    }

    pub unsafe fn set_vec4(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        gl::Uniform4f(self.get_uniform_location(name), x, y, z, w);
    }

    pub unsafe fn set_mat4(&self, name: &str, mat: &glm::Mat4) {
        gl::UniformMatrix4fv(self.get_uniform_location(name), 1, gl::FALSE, mat.as_ptr());
    }
}

pub struct ShaderBuilder {
    program_id: u32,
    shaders: Vec<u32>,
}

#[allow(dead_code)]
pub enum ShaderType {
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
}

impl Into<gl::types::GLenum> for ShaderType {
    fn into(self) -> gl::types::GLenum {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::TessellationControl => gl::TESS_CONTROL_SHADER,
            ShaderType::TessellationEvaluation => gl::TESS_EVALUATION_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
        }
    }
}

impl ShaderType {
    fn from_ext(ext: &std::ffi::OsStr) -> Result<ShaderType, String> {
        match ext.to_str().expect("Failed to read extension") {
            "vert" => Ok(ShaderType::Vertex),
            "frag" => Ok(ShaderType::Fragment),
            "tcs" => Ok(ShaderType::TessellationControl),
            "tes" => Ok(ShaderType::TessellationEvaluation),
            "geom" => Ok(ShaderType::Geometry),
            e => Err(e.to_string()),
        }
    }
}

impl ShaderBuilder {
    pub unsafe fn new() -> ShaderBuilder {
        ShaderBuilder {
            program_id: gl::CreateProgram(),
            shaders: vec![],
        }
    }

    pub unsafe fn attach_file(self, shader_path: &str) -> ShaderBuilder {
        let path = Path::new(shader_path);
        if let Some(extension) = path.extension() {
            let shader_type =
                ShaderType::from_ext(extension).expect("Failed to parse file extension.");
            let shader_src = std::fs::read_to_string(path)
                .expect(&format!("Failed to read shader source. {}", shader_path));
            self.compile_shader(&shader_src, shader_type)
        } else {
            panic!(
                "Failed to read extension of file with path: {}",
                shader_path
            );
        }
    }

    pub unsafe fn compile_shader(
        mut self,
        shader_src: &str,
        shader_type: ShaderType,
    ) -> ShaderBuilder {
        let shader = gl::CreateShader(shader_type.into());
        let c_str_shader = CString::new(shader_src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str_shader.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        if !self.check_shader_errors(shader) {
            panic!("Shader failed to compile.");
        }

        self.shaders.push(shader);

        self
    }

    unsafe fn check_shader_errors(&self, shader_id: u32) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetShaderInfoLog(
                shader_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "ERROR::Shader Compilation Failed!\n{}",
                String::from_utf8_lossy(&info_log)
            );
            return false;
        }
        true
    }

    unsafe fn check_linker_errors(&self) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetProgramiv(self.program_id, gl::LINK_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetProgramInfoLog(
                self.program_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            return false;
        }
        true
    }

    pub unsafe fn link(self) -> Shader {
        for &shader in &self.shaders {
            gl::AttachShader(self.program_id, shader);
        }
        gl::LinkProgram(self.program_id);

        // todo:: use this to make safer abstraction
        self.check_linker_errors();

        for &shader in &self.shaders {
            gl::DeleteShader(shader);
        }

        Shader {
            program_id: self.program_id,
        }
    }
}
