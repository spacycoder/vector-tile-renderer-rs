use glutin::event::VirtualKeyCode::{self, *};
use std::sync::{MutexGuard};

pub struct Player {
    forward: glm::Vec3,
    right: glm::Vec3,
    up: glm::Vec3,
    pub position: glm::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub movement_speed_multiplier: f32,
}

impl Player {
    pub fn new() -> Player {
        let yaw: f32 = -90.0;
        let pitch: f32 = 0.0;

        let position = glm::vec3(0.0, 0.0, 0.0);

        let dir_x = yaw.to_radians().cos() * pitch.to_radians().cos();
        let dir_y = pitch.to_radians().sin();
        let dir_z = yaw.to_radians().sin() * pitch.to_radians().cos();
        let forward = glm::normalize(&glm::vec3(dir_x as f32, dir_y as f32, dir_z as f32));

        let cam_right = forward.cross(&glm::vec3(0.0, 1.0, 0.0)).normalize();
        let cam_up = cam_right.cross(&forward).normalize();
        Player {
            position,
            forward,
            right: cam_right,
            up: cam_up,
            yaw,
            pitch,
            movement_speed: 10.0,
            mouse_sensitivity: 0.1,
            movement_speed_multiplier: 1.0,
        }
    }

    pub fn get_transform(&self) -> glm::Mat4 {
        let rotation = glm::mat4(
            self.right.x,
            self.right.y,
            self.right.z,
            0.0,
            self.up.x,
            self.up.y,
            self.up.z,
            0.0,
            -self.forward.x,
            -self.forward.y,
            -self.forward.z,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        let translation = glm::translate(&glm::identity(), &self.position);
        translation * glm::inverse(&rotation)
    }

    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32) {
        let xoffset = delta_x * self.mouse_sensitivity;
        let yoffset = delta_y * self.mouse_sensitivity;
        self.yaw += xoffset;
        self.pitch -= yoffset;
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }
        let dir_x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        let dir_y = self.pitch.to_radians().sin();
        let dir_z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.forward = glm::normalize(&glm::vec3(dir_x as f32, dir_y as f32, dir_z as f32));
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.forward = self.forward.normalize();
        self.right = self.forward.cross(&glm::vec3(0.0, 1.0, 0.0)).normalize();
        self.up = self.right.cross(&self.forward).normalize();
    }

    pub fn process_keyboard(
        &mut self,
        keys: MutexGuard<std::vec::Vec<glutin::event::VirtualKeyCode>>,
        delta_time: f32,
    ) {
        let mut shift_pressed = false;
        for key in keys.iter() {
            match key {
                VirtualKeyCode::A => {
                    self.position -= self.right
                        * self.movement_speed
                        * delta_time
                        * self.movement_speed_multiplier;
                }
                VirtualKeyCode::D => {
                    self.position += self.right
                        * self.movement_speed
                        * delta_time
                        * self.movement_speed_multiplier;
                }
                VirtualKeyCode::W => {
                    self.position += self.movement_speed
                        * self.forward
                        * delta_time
                        * self.movement_speed_multiplier;
                }
                VirtualKeyCode::S => {
                    self.position -= self.movement_speed
                        * self.forward
                        * delta_time
                        * self.movement_speed_multiplier;
                }
                VirtualKeyCode::E => {
                    self.position += self.movement_speed * self.up * delta_time * self.movement_speed_multiplier;
                }
                VirtualKeyCode::Q => {
                    self.position -= self.movement_speed * self.up * delta_time * self.movement_speed_multiplier;
                }
                VirtualKeyCode::Left => {
                    self.yaw -= self.movement_speed * delta_time;
                }
                VirtualKeyCode::Right => {
                    self.yaw += self.movement_speed * delta_time;
                }
                VirtualKeyCode::Up => {
                    self.pitch += self.movement_speed * delta_time;
                }
                VirtualKeyCode::Down => {
                    self.pitch -= self.movement_speed * delta_time;
                }
                VirtualKeyCode::LShift => {
                    shift_pressed = true;
                }
                _ => {}
            }
            if shift_pressed {
                self.movement_speed_multiplier = 4.0;
            } else {
                self.movement_speed_multiplier = 1.0;
            }
        }

        self.forward = self.forward.normalize();
        self.right = self.forward.cross(&glm::vec3(0.0, 1.0, 0.0)).normalize();
        self.up = self.right.cross(&self.forward).normalize();
        self.position = glm::vec3(self.position.x, self.position.y, self.position.z);
    }
}
