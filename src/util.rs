use std::ffi::CString;
extern crate nalgebra_glm as glm;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{mem, os::raw::c_void};

// allocate object and use the address as a random number generator since i'm not allowed to add packages.
pub fn rand_num_hack(max_number: i32) -> i32 {
    let num1 = vec![2, 3];
    let address1 = &num1 as *const Vec<i32>;
    address1 as i32 % max_number
}

// use time as random generator since i'm not allowed to add packages.
pub fn rand_num_hack2(max_number: u32) -> u32 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    time % max_number
}

// Helper functions to make interacting with OpenGL a little bit prettier. You will need these!
// The names should be pretty self explanatory
pub fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
pub fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
pub fn size_of<T>() -> usize {
    mem::size_of::<T>()
}

// Get an offset in bytes for n units of type T
pub fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

pub unsafe fn get_gl_string(name: gl::types::GLenum) -> String {
    std::ffi::CStr::from_ptr(gl::GetString(name) as *mut i8)
        .to_string_lossy()
        .to_string()
}

// Debug callback to panic upon enountering any OpenGL error
pub extern "system" fn debug_callback(
    source: u32,
    e_type: u32,
    id: u32,
    severity: u32,
    _length: i32,
    msg: *const i8,
    _data: *mut std::ffi::c_void,
) {
    if e_type != gl::DEBUG_TYPE_ERROR {
        return;
    }
    if severity == gl::DEBUG_SEVERITY_HIGH
        || severity == gl::DEBUG_SEVERITY_MEDIUM
        || severity == gl::DEBUG_SEVERITY_LOW
    {
        let severity_string = match severity {
            gl::DEBUG_SEVERITY_HIGH => "high",
            gl::DEBUG_SEVERITY_MEDIUM => "medium",
            gl::DEBUG_SEVERITY_LOW => "low",
            _ => "unknown",
        };
        unsafe {
            let string = CString::from_raw(msg as *mut i8);
            let error_message = String::from_utf8_lossy(string.as_bytes()).to_string();
            panic!(
                "{}: Error of severity {} raised from {}: {}\n",
                id, severity_string, source, error_message
            );
        }
    }
}

pub trait VecExt {
    fn add_vertex(&mut self, x: f32, y: f32, z: f32);
    fn num_vertices(&self) -> usize;
    fn get_vertex(&self, i: usize) -> (f32, f32, f32);
}

impl VecExt for Vec<f32> {
    fn add_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.push(x);
        self.push(y);
        self.push(z);
    }

    fn num_vertices(&self) -> usize {
        self.len() / 3
    }

    fn get_vertex(&self, i: usize) -> (f32, f32, f32) {
        (self[i * 3], self[i * 3 + 1], self[i * 3 + 2])
    }
}

pub fn calculate_normals(vertices: &Vec<glm::Vec3>, indices: &Vec<u32>) -> Vec<glm::Vec3> {
    let mut vertex_normals: Vec<glm::Vec3> = vec![glm::vec3(0.0, 0.0, 0.0); vertices.len()];
    let mut index = 0;
    if vertices.len() < 3 {
        return vertex_normals;
    }
    while index < indices.len() {
        let vertex_1 = indices[index];
        let vertex_2 = indices[index + 1];
        let vertex_3 = indices[index + 2];

        let edge_12 = vertices[vertex_2 as usize] - vertices[vertex_1 as usize];
        let edge_13 = vertices[vertex_3 as usize] - vertices[vertex_1 as usize];

        let area_weighted_normal = glm::cross(&edge_12, &edge_13);

        vertex_normals[vertex_1 as usize] += area_weighted_normal;
        vertex_normals[vertex_2 as usize] += area_weighted_normal;
        vertex_normals[vertex_3 as usize] += area_weighted_normal;
        index += 3;
    }

    index = 0;
    while index < vertex_normals.len() {
        vertex_normals[index] = vertex_normals[index].normalize();
        index += 1;
    }

    vertex_normals
}
