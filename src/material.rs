use std::collections::HashMap;
use super::shader::Shader;

#[derive(Clone)]
pub struct Material {
    shader: Shader,
    floats: HashMap<&'static str, f32>,
    vec2s: HashMap<&'static str, glm::Vec2>,
    vec3s: HashMap<&'static str, glm::Vec3>,
    vec4s: HashMap<&'static str, glm::Vec4>,
    mat4s: HashMap<&'static str, glm::Mat4>,
}

impl Material {
    pub fn new(shader: Shader) -> Material {
        Material{
            shader,
            floats: HashMap::new(),
            vec2s: HashMap::new(),
            vec3s: HashMap::new(),
            vec4s: HashMap::new(),
            mat4s: HashMap::new(),
        }
    }

    pub fn get_shader(&self) -> &Shader {
        &self.shader
    }

    pub fn set_float(&mut self, name: &'static str, value: f32) {
        self.floats.insert(name, value);
    }

    pub fn set_vec2(&mut self, name: &'static str, x: f32, y: f32) {
        self.vec2s.insert(name, glm::vec2(x, y));
    }

    pub fn set_vec3(&mut self, name: &'static str, x: f32, y: f32, z: f32) {
        self.vec3s.insert(name, glm::vec3(x, y, z));
    }

    pub fn set_vec4(&mut self, name: &'static str, x: f32, y: f32, z: f32, w: f32) {
        self.vec4s.insert(name, glm::vec4(x, y, z, w));
    }

    pub fn set_mat4(&mut self, name: &'static str, mat: &glm::Mat4) {
        self.mat4s.insert(name, mat.clone());
    }

    pub unsafe fn apply_values(&self) {
        self.shader.activate();
        for (key, value) in &self.floats {
           self.shader.set_float(key, *value);
        } 

        for (key, value) in &self.vec2s {
            self.shader.set_vec2(key, value.x, value.y);
        }

        for (key, value) in &self.vec3s {
            self.shader.set_vec3(key, value.x, value.y, value.z);
        }
        
        for (key, value) in &self.vec4s {
            self.shader.set_vec4(key, value.x, value.y, value.z, value.w);
        }

        for (key, value) in &self.mat4s {
            self.shader.set_mat4(key, value);
        }
    }
}

