use super::model;
use super::util;
use gl::types::*;
use image::{ColorType, GenericImageView};
use std::path::Path;
use std::ptr;

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<glm::Vec3>,
    pub indices: Vec<u32>,
    pub vao: u32,
    pub ebo: u32,
    pub vert_vbo: u32,
    pub uv_vbo: u32,
    pub uvs: Vec<glm::Vec2>,
    pub texture0: u32,
    pub normals_vbo: u32,
    pub normals: Vec<glm::Vec3>,
    pub img: TextureType,
}

#[derive(Clone)]
pub enum TextureType {
    Img(image::DynamicImage),
    FlipBook {
        columns: u32,
        rows: u32,
        img: image::DynamicImage,
        color: ColorType,
    },
    None,
}

impl Mesh {
    pub fn new(vertices: Vec<model::Vector3>, indices: Vec<u32>) -> Mesh {
        let mut mesh = Mesh {
            vertices,
            indices,
            vao: 0,
            vert_vbo: 0,
            ebo: 0,
            normals_vbo: 0,
            normals: Vec::new(),
            uv_vbo: 0,
            uvs: Vec::new(),
            texture0: 0,
            img: TextureType::None,
        };

        unsafe { mesh.setup_vao() }
        mesh
    }

    pub fn new_full(
        vertices: Vec<model::Vector3>,
        indices: Vec<u32>,
        normals: Vec<glm::Vec3>,
        uvs: Vec<glm::Vec2>,
    ) -> Mesh {
        let mut mesh = Mesh {
            vertices,
            indices,
            vao: 0,
            vert_vbo: 0,
            ebo: 0,
            normals_vbo: 0,
            normals,
            uv_vbo: 0,
            uvs,
            texture0: 0,
            img: TextureType::None,
        };

        unsafe { mesh.setup_vao_full() }
        mesh
    }


    pub fn set_flip_book_texture(&mut self, path: &str, columns: u32, rows: u32) {
        let img = image::open(&Path::new(path)).expect("Failed to load texture");
        let color = img.color();
        self.img = TextureType::FlipBook {
            columns,
            rows,
            img,
            color,
        };

        self.apply_texture();
    }

    pub fn set_texture(&mut self, path: &str) {
        let img = image::open(&Path::new(path)).expect("Failed to load texture");
        self.img = TextureType::Img(img);
        self.apply_texture();
    }

    pub fn apply_texture(&mut self) {
        if let TextureType::None = self.img {
            return;
        }

        let (data, width, height, color) = match &self.img {
            TextureType::FlipBook { img, color, .. } => (
                img.to_bytes().clone(),
                img.width(),
                img.height(),
                color_type(color),
            ),
            TextureType::Img(img) => (
                img.to_bytes().clone(),
                img.width(),
                img.height(),
                color_type(&img.color()),
            ),
            _ => (Vec::new(), 0, 0, gl::RGB),
        };

        unsafe {
            if self.texture0 == 0 {
                gl::GenTextures(1, &mut self.texture0);
            }
            gl::BindVertexArray(self.vao);
            gl::BindTexture(gl::TEXTURE_2D, self.texture0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            /*  gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32); */
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                color as i32,
                width as i32,
                height as i32,
                0,
                color,
                gl::UNSIGNED_BYTE,
                util::pointer_to_array(&data),
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }

    pub fn set_uvs(&mut self, uvs: Vec<glm::Vec2>) {
        if uvs.len() == 0 {
            return;
        }
        self.uvs = uvs;

        unsafe {
            if self.uv_vbo == 0 {
                gl::GenBuffers(1, &mut self.uv_vbo);
            }
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.uv_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                util::byte_size_of_array(&self.uvs) as GLsizeiptr,
                util::pointer_to_array(&self.uvs),
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                2 * util::size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );

            gl::EnableVertexAttribArray(2);
        }
    }

    pub fn set_normals(&mut self, normals: Vec<glm::Vec3>) {
        if normals.len() == 0 {
            return;
        }
        self.normals = normals;

        unsafe {
            if self.normals_vbo == 0 {
                gl::GenBuffers(1, &mut self.normals_vbo);
            }
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.normals_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                util::byte_size_of_array(&self.normals) as GLsizeiptr,
                util::pointer_to_array(&self.normals),
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * util::size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );

            gl::EnableVertexAttribArray(3);
        }
    }

    pub fn set_uvs_from_f32(&mut self, uvs: Vec<f32>) {
        if uvs.len() == 0 {
            return;
        }

        let mut uvs2: Vec<glm::Vec2> = Vec::new();
        let mut i = 0;
        while i < uvs.len() {
            let u = uvs[i];
            let v = uvs[i + 1];

            uvs2.push(glm::vec2(u, v));
            i += 2;
        }
        self.set_uvs(uvs2);
    }

    pub fn update_vao(&mut self) {
        unsafe { self.setup_vao() }
    }

    unsafe fn setup_vao(&mut self) {
        gl::GenVertexArrays(1, &mut self.vao);
        gl::GenBuffers(1, &mut self.vert_vbo);
        gl::GenBuffers(1, &mut self.ebo);
        gl::BindVertexArray(self.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vert_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            util::byte_size_of_array(&self.vertices) as GLsizeiptr,
            util::pointer_to_array(&self.vertices),
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            util::byte_size_of_array(&self.indices) as GLsizeiptr,
            util::pointer_to_array(&self.indices),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * util::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
    }

    unsafe fn setup_vao_full(&mut self) {
        gl::GenVertexArrays(1, &mut self.vao);
        gl::GenBuffers(1, &mut self.vert_vbo);
        gl::GenBuffers(1, &mut self.uv_vbo);
        gl::GenBuffers(1, &mut self.ebo);
        gl::GenBuffers(1, &mut self.normals_vbo);

        gl::BindVertexArray(self.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vert_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            util::byte_size_of_array(&self.vertices) as GLsizeiptr,
            util::pointer_to_array(&self.vertices),
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            util::byte_size_of_array(&self.indices) as GLsizeiptr,
            util::pointer_to_array(&self.indices),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * util::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, self.uv_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            util::byte_size_of_array(&self.uvs) as GLsizeiptr,
            util::pointer_to_array(&self.uvs),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            2 * util::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );

        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ARRAY_BUFFER, self.normals_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            util::byte_size_of_array(&self.normals) as GLsizeiptr,
            util::pointer_to_array(&self.normals),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            3,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * util::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );

        gl::EnableVertexAttribArray(3);
    }
}

fn color_type(color: &ColorType) -> u32 {
    match color {
        ColorType::Rgb8 => gl::RGB,
        /// Pixel is 8-bit RGB with an alpha channel
        ColorType::Rgba8 => gl::RGBA,
        _ => gl::RGB,
    }
}
