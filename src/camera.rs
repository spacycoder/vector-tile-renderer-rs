pub struct Camera {
    forward: glm::Vec3,
    right: glm::Vec3,
    up: glm::Vec3,
    radius: f32,
    yaw: f32, 
    pitch: f32,
    pub transform: glm::Mat4,
    pub position: glm::Vec3,
    pub parent_transform: glm::Mat4,
}

impl Camera {
    pub fn new(yaw: f32, pitch: f32, radius: f32) -> Camera {
        let dir_x = yaw.to_radians().cos() * pitch.to_radians().cos();
        let dir_y = pitch.to_radians().sin();
        let dir_z = yaw.to_radians().sin() * pitch.to_radians().cos();
        let forward = glm::normalize(&glm::vec3(dir_x as f32, dir_y as f32, dir_z as f32));

        let right = forward.cross(&glm::vec3(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(&forward).normalize();

        let transform = glm::mat4(
            right.x, right.y, right.z, 0.0, up.x, up.y, up.z, 0.0, -forward.x, -forward.y,
            -forward.z, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        let position = -forward * radius;

        Camera {
            radius,
            yaw,
            pitch,
            forward,
            right,
            up,
            transform,
            position,
            parent_transform: glm::identity(),
        }
    }
    pub fn handle_zoom(&mut self, delta: f32) {
        let new_radius = self.radius - delta * 8.0;
        if new_radius < 2.0 {
            return;
        }
        self.radius = new_radius;
        self.position = -self.forward * self.radius;
    }

    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32) {
        let xoffset = delta_x * 0.1;
        let yoffset = delta_y * 0.1;
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
        self.forward = glm::vec3(dir_x as f32, dir_y as f32, dir_z as f32).normalize();
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        self.right = self.forward.cross(&glm::vec3(0.0, 1.0, 0.0)).normalize();
        self.up = self.right.cross(&self.forward).normalize();
        self.position = -self.forward * self.radius;
    }

    pub fn get_view_transform(&self) -> glm::Mat4 {
        let cam_rotation = glm::mat4(
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

        let view_transform =
            glm::translate(&cam_rotation, &-self.position) * glm::inverse(&self.parent_transform);
        view_transform
    }
}
