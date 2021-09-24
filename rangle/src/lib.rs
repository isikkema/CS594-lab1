use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glam::{Mat4, Vec2, Vec4, Vec4Swizzles};

pub use camera::Camera;
pub use frame_buffer::FrameBuffer;
pub use error::RangleError;
pub use model::Model;
pub use rangle_display::RangleDisplay;
pub use shader::{Shader, ShaderType, ShaderTypeMap};

mod camera;
mod frame_buffer;
pub mod rangle_display;
pub mod error;
mod model;
mod shader;

pub enum RangleMode {
    Triangles,
    Lines,
    Points,
}

pub struct Rangle {
    display: Box<dyn RangleDisplay>,
    display_mode: RangleMode,
    models: Vec<(Rc<RefCell<Model>>, Rc<RefCell<Shader>>)>,
    frame_buffer: FrameBuffer,
    depth_buffer: Vec<f32>,
}

impl Rangle {
    pub fn new(display: Box<dyn RangleDisplay>) -> Result<Self, RangleError> {
        let display_mode = RangleMode::Triangles;
        let models = vec![];
        let (w, h) = display.get_size();
        let size = w as usize * h as usize;
        let frame_buffer = FrameBuffer::new(w, h)?;
        let depth_buffer = vec![-2.0; size];

        Ok(Rangle {
            display,
            display_mode,
            models,
            frame_buffer,
            depth_buffer,
        })
    }

    pub fn get_size(&self) -> (u16, u16) {
        self.display.get_size()
    }

    pub fn set_display_mode(&mut self, mode: RangleMode) {
        self.display_mode = mode;
    }

    pub fn compute_projection_matrix(
        fov: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    ) -> Mat4 {
        Mat4::perspective_rh(fov, aspect_ratio, z_near, z_far)
    }

    fn transform_coords_framebuffer_to_barycentric(
        px: u16,
        py: u16,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        x3: u16,
        y3: u16,
    ) -> (f32, f32, f32) {
        let (px, py) = (px as f32, py as f32);
        let (x1, y1) = (x1 as f32, y1 as f32);
        let (x2, y2) = (x2 as f32, y2 as f32);
        let (x3, y3) = (x3 as f32, y3 as f32);

        if px == x1 && py == y1 && px == x2 && py == y2 && px == x3 && py == y3 {
            return (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
        } else if px == x1 && py == y1 && px == x2 && py == y2 {
            return (0.5, 0.5, 0.0);
        } else if px == x1 && py == y1 && px == x3 && py == y3 {
            return (0.5, 0.0, 0.5);
        } else if px == x2 && py == y2 && px == x3 && py == y3 {
            return (0.0, 0.5, 0.5);
        } else {
            let mut a = (y2 - y3) * (px - x3) + (x3 - x2) * (py - y3);
            if a != 0.0 {
                a /= (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
            }

            let mut b = (y3 - y1) * (px - x3) + (x1 - x3) * (py - y3);
            if b != 0.0 {
                b /= (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
            }

            let c = 1.0 - a - b;

            return (a, b, c);
        }
    }

    fn interpolate_line(
        val1: &ShaderTypeMap,
        val2: &ShaderTypeMap,
        k: f32,
    ) -> HashMap<&'static str, ShaderType> {
        if !(val1.len() == val2.len() && val1.keys().all(|&key| val2.contains_key(key))) {
            panic!("vals must have the same keyset");
        }

        let mut rv = HashMap::new();
        let mut v2;
        for (&key, &v1) in val1 {
            v2 = val2[key];
            match (v1, v2) {
                (ShaderType::Vec3(v1), ShaderType::Vec3(v2)) => {
                    let new_v = v1 * (1.0 - k) + v2 * k;

                    rv.insert(key, ShaderType::Vec3(new_v));
                }
                (ShaderType::Vec4(v1), ShaderType::Vec4(v2)) => {
                    let new_v = v1 * (1.0 - k) + v2 * k;

                    rv.insert(key, ShaderType::Vec4(new_v));
                }
                _ => {
                    unimplemented!()
                }
            }
        }

        rv
    }

    fn interpolate_triangle(
        a: f32,
        b: f32,
        c: f32,
        val1: &ShaderTypeMap,
        val2: &ShaderTypeMap,
        val3: &ShaderTypeMap,
    ) -> ShaderTypeMap {
        if !(val1.len() == val2.len()
            && val1.len() == val3.len()
            && val1
                .keys()
                .all(|&key| val2.contains_key(key) && val3.contains_key(key)))
        {
            panic!("vals must have the same keyset");
        }

        let mut rv = HashMap::new();
        let mut v2;
        let mut v3;
        for (&key, &v1) in val1 {
            v2 = val2[key];
            v3 = val3[key];
            match (v1, v2, v3) {
                (ShaderType::Vec3(v1), ShaderType::Vec3(v2), ShaderType::Vec3(v3)) => {
                    let new_v = v1 * a + v2 * b + v3 * c;

                    rv.insert(key, ShaderType::Vec3(new_v));
                }
                (ShaderType::Vec4(v1), ShaderType::Vec4(v2), ShaderType::Vec4(v3)) => {
                    let new_v = v1 * a + v2 * b + v3 * c;

                    rv.insert(key, ShaderType::Vec4(new_v));
                }
                _ => {
                    unimplemented!()
                }
            }
        }

        rv
    }

    fn transform_coords_normalized_to_framebuffer(&self, x: f32, y: f32) -> (u16, u16) {
        let (width, height) = self.display.get_size();

        let fx = (((-x + 1.0) / 2.0) * (width - 1) as f32).round() as u16;
        let fy = (((-y + 1.0) / 2.0) * (height - 1) as f32).round() as u16;

        (
            fx.max(0).min(width-1),
            fy.max(0).min(height-1)
        )
    }

    fn framebuffer_draw_point(&mut self, x: u16, y: u16, val: Vec4) {
        let (width, height) = self.display.get_size();
        if x >= width || y >= height {
            return;
        }

        let (x, y) = (x as usize, y as usize);

        let color_vec = (val
            .max(Vec4::new(0.0, 0.0, 0.0, 0.0))
            .min(Vec4::new(1.0, 1.0, 1.0, 1.0))
            * 255.0)
            .round();

        let color = (
            color_vec.x as u8,
            color_vec.y as u8,
            color_vec.z as u8,
            color_vec.w as u8,
        );

        self.frame_buffer.draw_point(x, y, color);
    }

    fn render_point(&mut self, p: Vec2, val: ShaderTypeMap, shader: &Shader) {
        let (width, _height) = self.get_size();
        let width = width as usize;
        let (fx, fy) = self.transform_coords_normalized_to_framebuffer(p.x, p.y);

        let z = match val.get("rangle_Position") {
            Some(&ShaderType::Vec4(v)) => -v.z / v.w,
            _ => panic!()
        };

        let color;
        let index = fy as usize * width + fx as usize;
        if z >= -1.0 && z <= 1.0 && z > self.depth_buffer[index] {
            self.depth_buffer[index] = z;
            color = shader.run_fragment(val);

            self.framebuffer_draw_point(fx, fy, color);
        }
    }

    fn render_line(
        &mut self,
        p1: Vec2,
        p2: Vec2,
        val1: ShaderTypeMap,
        val2: ShaderTypeMap,
        shader: &Shader,
    ) {
        let (mut x1, mut y1) = (p1.x, p1.y);
        let (mut x2, mut y2) = (p2.x, p2.y);

        let (mut val1, mut val2) = (val1, val2);

        let xd = (x1 - x2).abs();
        let yd = (y1 - y2).abs();

        let (width, _height) = self.get_size();
        let width = width as usize;

        let mut k = 0.0;
        let mut index;
        let mut color;
        let mut interpolated_val;

        if xd >= yd {
            if x2 > x1 {
                let temp = x1;
                x1 = x2;
                x2 = temp;

                let temp = y1;
                y1 = y2;
                y2 = temp;

                let temp = val1;
                val1 = val2;
                val2 = temp;
            }

            let (fx1, fy1) = self.transform_coords_normalized_to_framebuffer(x1, y1);
            let (fx2, fy2) = self.transform_coords_normalized_to_framebuffer(x2, y2);

            let fxd = (fx2 - fx1) as f32;
            let fyd = fy2 as f32 - fy1 as f32;

            let dist_jump = 1.0 / fxd;
            let y_jump = fyd / fxd;

            let mut fy = fy1 as f32;
            for fx in fx1..=fx2 {
                interpolated_val = Self::interpolate_line(&val1, &val2, k);
                let z = match interpolated_val.get("rangle_Position") {
                    Some(&ShaderType::Vec4(v)) => -v.z / v.w,
                    _ => panic!()
                };

                index = fy as usize * width + fx as usize;
                if z >= -1.0 && z <= 1.0 && z > self.depth_buffer[index] {
                    self.depth_buffer[index] = z;
                    color = shader.run_fragment(interpolated_val);

                    self.framebuffer_draw_point(fx, fy.round() as u16, color);
                }

                fy += y_jump;
                k += dist_jump;
            }
        } else {
            if y2 > y1 {
                let temp = x1;
                x1 = x2;
                x2 = temp;

                let temp = y1;
                y1 = y2;
                y2 = temp;

                let temp = val1;
                val1 = val2;
                val2 = temp;
            }

            let (fx1, fy1) = self.transform_coords_normalized_to_framebuffer(x1, y1);
            let (fx2, fy2) = self.transform_coords_normalized_to_framebuffer(x2, y2);

            let fxd = fx2 as f32 - fx1 as f32;
            let fyd = (fy2 - fy1) as f32;

            let dist_jump = 1.0 / fyd;
            let x_jump = fxd / fyd;

            let mut fx = fx1 as f32;
            for fy in fy1..=fy2 {
                interpolated_val = Self::interpolate_line(&val1, &val2, k);
                let z = match interpolated_val.get("rangle_Position") {
                    Some(&ShaderType::Vec4(v)) => -v.z / v.w,
                    _ => panic!()
                };

                index = fy as usize * width + fx as usize;
                if z >= -1.0 && z <= 1.0 && z > self.depth_buffer[index] {
                    self.depth_buffer[index] = z;
                    color = shader.run_fragment(interpolated_val);

                    self.framebuffer_draw_point(fx.round() as u16, fy, color);
                }

                fx += x_jump;
                k += dist_jump;
            }
        }
    }

    fn render_triangle(
        &mut self,
        p1: Vec2,
        val1: ShaderTypeMap,
        p2: Vec2,
        val2: ShaderTypeMap,
        p3: Vec2,
        val3: ShaderTypeMap,
        shader: &Shader,
    ) {
        let (x1, y1) = (p1.x, p1.y);
        let (x2, y2) = (p2.x, p2.y);
        let (x3, y3) = (p3.x, p3.y);

        let (fx1, fy1) = self.transform_coords_normalized_to_framebuffer(x1, y1);
        let (fx2, fy2) = self.transform_coords_normalized_to_framebuffer(x2, y2);
        let (fx3, fy3) = self.transform_coords_normalized_to_framebuffer(x3, y3);

        let fx_min = fx1.min(fx2).min(fx3) as u16;
        let fx_max = fx1.max(fx2).max(fx3) as u16;
        let fy_min = fy1.min(fy2).min(fy3) as u16;
        let fy_max = fy1.max(fy2).max(fy3) as u16;

        let (width, _height) = self.get_size();
        let width = width as usize;

        let mut color;
        let mut interpolated_val;
        let mut index;
        for fy in fy_min..=fy_max {
            for fx in fx_min..=fx_max {
                let (a, b, c) = Self::transform_coords_framebuffer_to_barycentric(
                    fx, fy, fx1, fy1, fx2, fy2, fx3, fy3,
                );

                if a >= 0.0 && a <= 1.0 && b >= 0.0 && b <= 1.0 && c >= 0.0 && c <= 1.0 {
                    interpolated_val = Self::interpolate_triangle(a, b, c, &val1, &val2, &val3);
                    let z = match interpolated_val.get("rangle_Position") {
                        Some(&ShaderType::Vec4(v)) => -v.z / v.w,
                        _ => panic!()
                    };

                    index = fy as usize * width + fx as usize;
                    if z >= -1.0 && z <= 1.0 && z > self.depth_buffer[index] {
                        self.depth_buffer[index] = z;
                        color = shader.run_fragment(interpolated_val);

                        self.framebuffer_draw_point(fx, fy, color);
                    }
                }
            }
        }
    }

    fn render_model(&mut self, model: &Model, shader: &Shader) {
        for i in (0..model.vertex_indices.len()).step_by(3) {
            let val1 = shader.run_vertex(i);
            let val2 = shader.run_vertex(i + 1);
            let val3 = shader.run_vertex(i + 2);

            let p1 = match val1.get("rangle_Position") {
                Some(&ShaderType::Vec4(v)) => v / v.w,
                _ => panic!("stop it"),
            };
            let p2 = match val2.get("rangle_Position") {
                Some(&ShaderType::Vec4(v)) => v / v.w,
                _ => panic!("stop it"),
            };
            let p3 = match val3.get("rangle_Position") {
                Some(&ShaderType::Vec4(v)) => v / v.w,
                _ => panic!("stop it"),
            };

            match self.display_mode {
                RangleMode::Triangles => {
                    self.render_triangle(p1.xy(), val1, p2.xy(), val2, p3.xy(), val3, shader);
                },
                RangleMode::Lines => {
                    self.render_line(p1.xy(), p2.xy(), val1.clone(), val2.clone(), shader);
                    self.render_line(p2.xy(), p3.xy(), val2, val3.clone(), shader);
                    self.render_line(p3.xy(), p1.xy(), val3, val1, shader);
                },
                RangleMode::Points => {
                    self.render_point(p1.xy(), val1, shader);
                    self.render_point(p2.xy(), val2, shader);
                    self.render_point(p3.xy(), val3, shader);
                }
            }
        }
    }

    pub fn add_model(&mut self, model: Model, shader: Rc<RefCell<Shader>>) -> Rc<RefCell<Model>> {
        let model = Rc::new(RefCell::new(model));

        self.models.push((model.clone(), shader));

        model
    }

    pub fn render_scene(&mut self) -> Result<(), RangleError> {
        let color = self.display.get_background_color();

        let _ = self.depth_buffer.iter_mut().map(|v| *v = -2.0).count();
        self.frame_buffer.fill_buffer(color);

        let models = self.models.clone();
        for (model, shader) in &models {
            let model = model.borrow();
            let shader = shader.borrow();

            self.render_model(&model, &shader);
        }

        self.display.draw_buffer(&self.frame_buffer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
