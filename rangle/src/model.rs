use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use glam::{Mat4, Quat, Vec3, EulerRot};

use crate::error::RangleError;

#[derive(Clone)]
pub struct Model {
    pub(crate) vertices: Vec<f32>,
    pub(crate) vertex_indices: Vec<usize>,
    center: (f32, f32, f32),
    scale: (f32, f32, f32),
    rotate: (f32, f32, f32),
    translate: (f32, f32, f32),
}

impl Model {
    pub fn from_file<'a>(filename: &'a str) -> Result<Self, RangleError> {
        let file = File::open(filename)?;

        let mut vertices = vec![];
        let mut vertex_indices = vec![];

        let reader = BufReader::new(file);

        let mut vx: f32;
        let mut vy: f32;
        let mut vz: f32;
        let mut index1: usize;
        let mut index2: usize;
        let mut index3: usize;
        for line in reader.lines() {
            let line = line?;

            let chunks: Vec<&str>;

            if line.starts_with("v ") {
                chunks = line
                    .strip_prefix("v ")
                    .unwrap()
                    .split(' ')
                    .filter(|&s| !s.is_empty())
                    .collect();

                // TODO: Stop unwrapping and return a real error
                vx = chunks[0].parse().unwrap();
                vy = chunks[1].parse().unwrap();
                vz = chunks[2].parse().unwrap();

                vertices.push(vx);
                vertices.push(vy);
                vertices.push(vz);
            } else if line.starts_with("f ") {
                chunks = line
                    .strip_prefix("f ")
                    .unwrap()
                    .split(' ')
                    .filter(|&s| !s.is_empty())
                    .collect();
                for i in 1..=chunks.len() - 2 {
                    index1 = chunks[0].split('/').nth(0).unwrap().parse().unwrap();
                    index2 = chunks[i].split('/').nth(0).unwrap().parse().unwrap();
                    index3 = chunks[i + 1].split('/').nth(0).unwrap().parse().unwrap();

                    vertex_indices.push(index1 - 1);
                    vertex_indices.push(index2 - 1);
                    vertex_indices.push(index3 - 1);
                }
            }
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;
        let v_len = (vertices.len() / 3) as f32;
        for i in (0..vertices.len()).step_by(3) {
            sum_x += vertices[i];
            sum_y += vertices[i + 1];
            sum_z += vertices[i + 2];
        }

        Ok(Model {
            vertices: vertices,
            vertex_indices: vertex_indices,
            center: (sum_x / v_len, sum_y / v_len, sum_z / v_len),
            scale: (1.0, 1.0, 1.0),
            translate: (0.0, 0.0, 0.0),
            rotate: (0.0, 0.0, 0.0),
        })
    }

    pub fn from_vectors(vertices: Vec<f32>, vertex_indices: Vec<usize>) -> Self {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;
        let v_len = (vertices.len() / 3) as f32;
        for i in (0..vertices.len()).step_by(3) {
            sum_x += vertices[i];
            sum_y += vertices[i + 1];
            sum_z += vertices[i + 2];
        }

        Model {
            vertices: vertices,
            vertex_indices: vertex_indices,
            center: (sum_x / v_len, sum_y / v_len, sum_z / v_len),
            scale: (1.0, 1.0, 1.0),
            translate: (0.0, 0.0, 0.0),
            rotate: (0.0, 0.0, 0.0),
        }
    }

    pub fn get_vertices(&self) -> Vec<f32> {
        self.vertices.clone()
    }

    pub fn get_vertex_buffer(&self) -> Vec<Vec3> {
        let mut rv = vec![];

        let mut i0;
        for i in 0..self.vertex_indices.len() {
            i0 = 3 * self.vertex_indices[i];

            rv.push(Vec3::new(
                self.vertices[i0],
                self.vertices[i0 + 1],
                self.vertices[i0 + 2],
            ));
        }

        rv
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = (x, y, z);
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32, roll: f32) {
        self.rotate = (yaw, pitch, roll);
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.translate = (x, y, z);
    }

    pub fn compute_model_matrix(&self) -> Mat4 {
        let center_translate = Mat4::from_translation(Vec3::from(self.center) * -1.0);
        let translate_rotate_scale = Mat4::from_scale_rotation_translation(
            self.scale.into(),
            Quat::from_euler(EulerRot::ZYX, self.rotate.2, self.rotate.0, self.rotate.1),
            self.translate.into(),
        );

        translate_rotate_scale * center_translate
    }
}
