use super::state;
use super::shader;
extern crate nalgebra_glm as glm;

pub trait Renderable {
    fn before_render(&mut self, state: &state::State);
    fn on_render(&mut self, s: &state::State, view_transform: &glm::Mat4, projection_transform: &glm::Mat4);
    fn shader_program(&self) -> &shader::Shader;
    fn get_transform(&self) -> &glm::Mat4;
}