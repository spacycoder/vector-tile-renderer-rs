use super::util;
use gl;
use gl::types::*;
use image;
use image::GenericImageView;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
pub unsafe fn create_skybox() -> (u32, u32) {
    let skybox_vertices: [f32; 108] = [
        // positions
        -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0,
        -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0,
        1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0,
        1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
    ];

    // skybox VAO
    let (mut skybox_vao, mut skybox_vbo) = (0, 0);
    gl::GenVertexArrays(1, &mut skybox_vao);
    gl::GenBuffers(1, &mut skybox_vbo);
    gl::BindVertexArray(skybox_vao);
    gl::BindBuffer(gl::ARRAY_BUFFER, skybox_vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        util::byte_size_of_array(&skybox_vertices) as GLsizeiptr,
        util::pointer_to_array(&skybox_vertices),
        gl::STATIC_DRAW,
    );
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * util::size_of::<GLfloat>() as GLsizei, ptr::null());

    let faces = [
        "assets/textures/skybox/right.png",
        "assets/textures/skybox/left.png",
        "assets/textures/skybox/top.png",
        "assets/textures/skybox/bottom.png",
        "assets/textures/skybox/back.png",
        "assets/textures/skybox/front.png",
    ];
    let cubemap_texture = load_cubemap(&faces);
    (skybox_vao, cubemap_texture)
}

unsafe fn load_cubemap(faces: &[&str]) -> u32 {
    let mut texture_id = 0;
    gl::GenTextures(1, &mut texture_id);
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture_id);

    for (i, face) in faces.iter().enumerate() {
        let img = image::open(&Path::new(face)).expect("Cubemap texture failed to load");
        let data = img.to_bytes();
        gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
            0,
            gl::RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            &data[0] as *const u8 as *const c_void,
        );
    }

    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_MIN_FILTER,
        gl::LINEAR as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_MAG_FILTER,
        gl::LINEAR as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_S,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_T,
        gl::CLAMP_TO_EDGE as i32,
    );
    gl::TexParameteri(
        gl::TEXTURE_CUBE_MAP,
        gl::TEXTURE_WRAP_R,
        gl::CLAMP_TO_EDGE as i32,
    );

    texture_id
}
