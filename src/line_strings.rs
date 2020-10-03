use super::mesh;
use super::polygons;
use super::util;

pub struct LineStringDecoder<'a> {
    geometry: &'a [u32],
    position: usize,
    scale: f32,
    x: i64,
    y: i64,
    command: u32,
    repeat_command: i32,
    extent: u32,
    pub output_mesh: polygons::OutputMesh,
    thickness: f32,
    height: f32
}

impl<'a> LineStringDecoder<'a> {
    pub fn new(extent: u32, geometry: &'a [u32], thickness: f32, height: f32) -> LineStringDecoder {
        LineStringDecoder {
            geometry,
            position: 0,
            scale: 1.0 / ((extent as f32) - 1.0),
            x: 0,
            y: 0,
            command: 1,
            repeat_command: 0,
            extent,
            output_mesh: polygons::OutputMesh::new(),
            thickness,
            height,
        }
    }

    pub fn decode(&mut self) {
        let mut line: Vec<glm::Vec2> = vec![];
        while self.position < self.geometry.len() {
            self.advance_command();
            let start_point = self.advance_cursor();
            line.push(start_point);
            self.advance_command();

            for _ in 0..self.repeat_command {
                line.push(self.advance_cursor());
            }

            let (vertices, indices, uvs, normals) = generate_path(&line, self.thickness, self.height);
            if vertices.len() >= 3 {
                let mut vertices = vertices;
                let mut uvs = uvs;
                let mut normals = normals;
                self.output_mesh
                    .add_elements(&mut vertices, &indices, &mut uvs, &mut normals);
            }
            line.clear();
        }
    }

    fn advance_command(&mut self) {
        let command_data = self.geometry[self.position];
        self.position += 1;
        // The 3 lowest bits of the command encode the type, the rest are the repeat count.
        self.command = command_data & 0x7;
        self.repeat_command = (command_data >> 3) as i32;
    }

    fn advance_cursor(&mut self) -> glm::Vec2 {
        // For each MoveTo and LineTo repetition there are 2 parameter integers.
        let param0 = self.geometry[self.position] as i64;
        self.position += 1;
        let param1 = self.geometry[self.position] as i64;
        self.position += 1;
        // The parameters are zigzag-encoded deltas for x and y of the cursor.
        self.x += (param0 >> 1) ^ (-(param0 & 1));
        self.y += (param1 >> 1) ^ (-(param1 & 1));
        // The coordinates are normalized and Y is flipped to match our expected axes.
        return glm::vec2((self.x as f32) * self.scale, (self.y as f32) * self.scale);
    }
}

pub fn generate_path(
    path: &Vec<glm::Vec2>,
    thickness: f32,
    height: f32,
) -> (Vec<glm::Vec3>, Vec<u32>, Vec<glm::Vec2>, Vec<glm::Vec3>) {
    let mut vertices: Vec<glm::Vec3> = Vec::new();
    let mut uvs: Vec<glm::Vec2> = Vec::new();
    let num_indices = (path.len() - 1) * 6;
    let mut indices: Vec<u32> = vec![];

    let mut vertex_index: u32 = 0;
    let length = path.len();

    let start_pos = path[0];
    let next_pos = path[1];

    let forward = next_pos - start_pos;
    let forward = glm::normalize(&forward);
    let left = glm::vec2(-forward.y, forward.x);

    let point_a = start_pos + left * thickness;
    let point_b = start_pos - left * thickness;

    vertices.push(glm::vec3(point_a.x, height, point_a.y));
    vertices.push(glm::vec3(point_b.x, height, point_b.y));

    for i in 1..length {
        let start_pos = path[i - 1];
        let next_pos = path[i];

        let forward = next_pos - start_pos;
        let forward = glm::normalize(&forward);
        let left = glm::vec2(-forward.y, forward.x);

        let point_a = start_pos + left * thickness;
        let point_b = start_pos - left * thickness;

        vertices.push(glm::vec3(point_a.x, height, point_a.y));
        vertices.push(glm::vec3(point_b.x, height, point_b.y));

        uvs.push(glm::vec2(i as f32 / length as f32, 0.0));
        uvs.push(glm::vec2(i as f32 / length as f32, 1.0));

        indices.push(vertex_index + 3);
        indices.push(vertex_index + 1);
        indices.push(vertex_index + 0);
        indices.push(vertex_index + 2);
        indices.push(vertex_index + 3);
        indices.push(vertex_index + 0);

        vertex_index += 2;
    }

    let normals = util::calculate_normals(&vertices, &indices);
    (vertices, indices, uvs, normals)
}

#[derive(Clone, Debug)]
pub struct LineOptions {
    pub width: f32,
    pub height: f32,
}

impl LineOptions {
    pub fn new(height: f32, width: f32) -> Self {
        LineOptions { height, width }
    }
}
