extern crate nalgebra_glm as glm;
use super::mesh::Mesh;
use super::renderable::Renderable;
use super::shader;
use super::state;
use gl::types::*;
use std::ptr;
use tobj;
use super::material;
pub type Vector3 = glm::Vec3;

pub struct Model {
    pub name: String,
    pub transform: glm::Mat4,
    pub world_transform: glm::Mat4,
    material: material::Material,
    pub meshes: Vec<Mesh>,
    before_render: Option<Box<dyn Fn(&glm::Mat4, &mut material::Material, &state::State) -> glm::Mat4>>,
}

impl Model {
    pub fn new(
        name: String,
        meshes: Vec<Mesh>,
        material: material::Material,
        transform: glm::Mat4,
        before_render: Option<Box<dyn Fn(&glm::Mat4, &mut material::Material, &state::State) -> glm::Mat4>>,
    ) -> Model {
        Model {
            name,
            meshes,
            transform,
            material,
            before_render,
            world_transform: glm::identity(),
        }
    }

    pub fn set_transform(&mut self, transform: glm::Mat4) {
        self.transform = transform;
    }
}

impl Renderable for Model {
    fn before_render(&mut self, state: &state::State) {
        if let Some(func) = &self.before_render {
            let transform = func(&self.transform, &mut self.material, state);
            self.transform = transform;
        }
    }

    fn on_render(
        &mut self,
        s: &state::State,
        view_transform: &glm::Mat4,
        projection_transform: &glm::Mat4,
    ) {
        unsafe {
            let model_transform = &self.world_transform;

            let transform: glm::Mat4 = projection_transform * view_transform * model_transform;

            self.material.get_shader().set_mat4("transform", &transform);
            self.material.get_shader().set_mat4("modelTransform", model_transform);
            self.material.get_shader().set_mat4("viewTransform", view_transform);
            self.material.get_shader()
                .set_mat4("projectionTransform", projection_transform);
            
            self.material.apply_values();
            for mesh in &self.meshes {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, mesh.texture0);

                gl::BindVertexArray(mesh.vao);
                gl::DrawElements(
                    gl::TRIANGLES,
                    mesh.indices.len() as GLsizei,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
    }

    fn shader_program(&self) -> &shader::Shader {
        self.material.get_shader()
    }

    fn get_transform(&self) -> &glm::Mat4 {
        &self.transform
    }
}