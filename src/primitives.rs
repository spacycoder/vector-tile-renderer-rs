use super::mesh::Mesh;

pub fn generate_quad() -> Mesh {
    let vertices: Vec<glm::Vec3> = vec![
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(1.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        glm::vec3(1.0, 1.0, 0.0),
    ];

    let indices = vec![0, 1, 2, 2, 1, 3];
    let uvs = vec![
        glm::vec2(0.0, 0.0),
        glm::vec2(1.0, 0.0),
        glm::vec2(0.0, 1.0),
        glm::vec2(1.0, 1.0),
    ];

    let mut m = Mesh::new(vertices, indices);
    m.set_uvs(uvs);
    m
}
