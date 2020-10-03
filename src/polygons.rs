use super::material::Material;
use super::mesh;
use super::util;

pub struct PolygonDecoder<'a> {
    geometry: &'a [u32],
    position: usize,
    scale: f32,
    x: i64,
    y: i64,
    command: u32,
    repeat_command: i32,
    extent: u32,
    polygon_builder: &'a mut PolygonBuilder,
}

impl<'a> PolygonDecoder<'a> {
    pub fn new(
        extent: u32,
        geometry: &'a [u32],
        polygon_builder: &'a mut PolygonBuilder,
    ) -> PolygonDecoder<'a> {
        PolygonDecoder {
            geometry,
            position: 0,
            scale: 1.0 / ((extent as f32) - 1.0),
            x: 0,
            y: 0,
            command: 1,
            repeat_command: 0,
            extent,
            polygon_builder: polygon_builder,
        }
    }

    pub fn decode(&mut self) {
        let mut ring: Vec<glm::Vec2> = vec![];
        let mut is_polygon_started = false;
        while self.position < self.geometry.len() {
            self.advance_command();
            let start_point = self.advance_cursor();
            ring.push(start_point);
            self.advance_command();

            for _ in 0..self.repeat_command {
                ring.push(self.advance_cursor());
            }
            ring.push(ring[0]);
            self.advance_command();

            let area = self.signed_area(&ring);
            if area > 0.0 {
                if is_polygon_started {
                    self.polygon_builder.on_end_polygon();
                }
                self.polygon_builder.on_begin_polygon();
                is_polygon_started = true;
            }

            self.polygon_builder.on_begin_linear_ring();
            for point in &ring {
                self.polygon_builder.on_point(point.clone());
            }
            ring.clear();
        }

        self.polygon_builder.on_end_polygon();
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

    fn signed_area(&self, ring: &Vec<glm::Vec2>) -> f32 {
        if ring.len() == 0 {
            return 0.0;
        }

        let mut area = 0.0;
        let mut prev = ring[ring.len() - 1];
        for curr in ring {
            area += curr.x * prev.y - curr.y * prev.x;
            prev = curr.clone();
        }
        return 0.5 * area;
    }
}

pub struct PolygonBuilder {
    coordinates: Vec<f32>,
    holes: Vec<usize>,
    points_in_ring: u32,
    points_in_polygon: u32,

    last_point: glm::Vec2,
    extrusion_vertices: Vec<glm::Vec3>,
    extrusion_uvs: Vec<glm::Vec2>,
    polygon_uvs: Vec<glm::Vec2>,
    extrusion_indices: Vec<u32>,
    u_coordinate_total: f32,
    options: PolygonOptions,
    pub output_mesh: OutputMesh,
}

impl PolygonBuilder {
    pub fn new(polygon_options: PolygonOptions) -> PolygonBuilder {
        PolygonBuilder {
            coordinates: vec![],
            holes: vec![],
            points_in_ring: 0,
            points_in_polygon: 0,
            last_point: glm::vec2(0.0, 0.0),
            extrusion_vertices: vec![],
            extrusion_uvs: vec![],
            polygon_uvs: vec![],
            extrusion_indices: vec![],
            u_coordinate_total: 0.0,
            options: polygon_options,
            output_mesh: OutputMesh::new(),
        }
    }

    pub fn add_uv(&mut self, uv: glm::Vec2) {
        self.polygon_uvs.push(uv);
    }

    pub fn on_point(&mut self, point: glm::Vec2) {
        let max_height = self.options.max_height;
        let min_height = self.options.min_height;

        if self.options.build_walls && self.points_in_ring > 0 {
            let p0 = self.last_point;
            let p1 = point;

            let index_offset = self.extrusion_vertices.len();
            // Increase the u coordinate by the 2D distance between the points.
            let u_coordinate_next =
                self.u_coordinate_total + glm::length(&glm::vec2(p1.x - p0.x, p1.y - p0.y));

            let v0 = glm::vec3(p0.x, max_height, p0.y);
            let v1 = glm::vec3(p1.x, max_height, p1.y);
            let v2 = glm::vec3(p0.x, min_height, p0.y);
            let v3 = glm::vec3(p1.x, min_height, p1.y);

            self.extrusion_vertices.push(v0);
            self.extrusion_vertices.push(v1);
            self.extrusion_vertices.push(v2);
            self.extrusion_vertices.push(v3);

            let v_bottom = 0.0;
            let v_top = 1.0;
            let u_left = 0.0;
            let u_right = 1.0;

            self.extrusion_uvs.push(glm::vec2(u_right, v_top));
            self.extrusion_uvs.push(glm::vec2(u_left, v_top));
            self.extrusion_uvs.push(glm::vec2(u_right, v_bottom));
            self.extrusion_uvs.push(glm::vec2(u_left, v_bottom));

            self.extrusion_indices.push((index_offset + 0) as u32);
            self.extrusion_indices.push((index_offset + 1) as u32);
            self.extrusion_indices.push((index_offset + 3) as u32);
            self.extrusion_indices.push((index_offset + 0) as u32);
            self.extrusion_indices.push((index_offset + 3) as u32);
            self.extrusion_indices.push((index_offset + 2) as u32);

            self.u_coordinate_total = u_coordinate_next;
        }

        self.last_point = point;

        self.coordinates.push(point.x);
        self.coordinates.push(point.y);

        self.points_in_ring += 1;
        self.points_in_polygon += 1;
    }

    pub fn on_begin_linear_ring(&mut self) {
        self.points_in_ring = 0;
        self.u_coordinate_total = 0.0;
        if self.points_in_polygon > 0 {
            self.holes.push(self.points_in_polygon as usize);
        }
    }

    pub fn on_begin_polygon(&mut self) {
        self.coordinates.clear();
        self.holes.clear();
        self.extrusion_vertices.clear();
        self.extrusion_uvs.clear();
        self.extrusion_indices.clear();
        self.polygon_uvs.clear();
        self.points_in_polygon = 0;
    }

    pub fn on_end_polygon(&mut self) {
        // First add vertices and indices for extrusions.
        if self.extrusion_vertices.len() > 0 {
            let mut verts = self.extrusion_vertices.clone();
            let mut uvs = self.extrusion_uvs.clone();
            let mut normals = util::calculate_normals(&verts, &self.extrusion_indices);
            self.output_mesh.add_elements(
                &mut verts,
                &self.extrusion_indices,
                &mut uvs,
                &mut normals,
            )
        }

        if self.coordinates.len() > 0 {
            let coordinates: Vec<f64> = self.coordinates.iter().map(|n| (*n as f64)).collect();

            let indices = earcutr::earcut(&coordinates, &self.holes, 2);
            let mut indices: Vec<u32> = indices.iter().map(|n| (*n as u32)).collect();

            let mut vertices: Vec<glm::Vec3> = vec![];

            let mut uvs = if self.polygon_uvs.len() > 0 {
                self.polygon_uvs.clone()
            } else {
                let mut uvs: Vec<glm::Vec2> = vec![];
                let mut i = 0;
                while i < coordinates.len() {
                    uvs.push(glm::vec2(coordinates[i] as f32, coordinates[i + 1] as f32));
                    i += 2;
                }
                uvs
            };

            let mut i = 0;
            while i < coordinates.len() {
                let v = glm::vec3(
                    coordinates[i] as f32,
                    self.options.max_height,
                    coordinates[i + 1] as f32,
                );
                vertices.push(v);
                i += 2
            }

            i = 0;
            // flip order
            while i < indices.len() {
                let index = indices[i];
                indices[i] = indices[i + 2];
                indices[i + 2] = index;
                i += 3;
            }

            let mut normals = util::calculate_normals(&vertices, &indices);
            self.output_mesh
                .add_elements(&mut vertices, &indices, &mut uvs, &mut normals)
        }
    }
}

#[derive(Clone, Debug)]
pub struct PolygonOptions {
    pub max_height: f32,
    pub min_height: f32,
    pub build_walls: bool,
}

pub struct OutputMesh {
    pub vertices: Vec<glm::Vec3>,
    pub indices: Vec<u32>,
    pub uvs: Vec<glm::Vec2>,
    pub normals: Vec<glm::Vec3>,
}

impl OutputMesh {
    pub fn new() -> Self {
        OutputMesh {
            vertices: vec![],
            indices: vec![],
            uvs: vec![],
            normals: vec![],
        }
    }

    pub fn add_elements(
        &mut self,
        vertices: &mut Vec<glm::Vec3>,
        indices: &Vec<u32>,
        uvs: &mut Vec<glm::Vec2>,
        normals: &mut Vec<glm::Vec3>,
    ) {
        let offset = self.vertices.len();
        self.vertices.append(vertices);
        self.normals.append(normals);
        self.uvs.append(uvs);

        for index in indices {
            self.indices.push(offset as u32 + index);
        }
    }
}
