
use glutin::event::{
    VirtualKeyCode::{self, *},
};

pub struct State {
    pub camera_position: glm::Vec3,
    pub elapsed: f32,
    pub delta_time: f32,
    pub frame_num: u32,
    pub pressed_keys: Vec<VirtualKeyCode>,
    pub delta_x: f32,
    pub delta_y: f32,
}