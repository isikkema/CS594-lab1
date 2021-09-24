use core::panic;
use std::{f32::consts::PI, time::{Duration, Instant}};

use crossterm::event;
use event::{Event, KeyCode};
use glam::Vec4;

use rangle::{Camera, Model, Rangle, RangleError, RangleMode, Shader, ShaderType};

fn main() -> Result<(), RangleError> {
    let mut time_avg = Duration::from_secs(0);
    let mut now;
    let mut i = 0;

    {
        let mut rangle = Rangle::new()?;

        let shader = Shader::new(
            |mut attributes, uniforms| {
                let position = match attributes["position"] {
                    ShaderType::Vec3(v) => v,
                    _ => {
                        panic!("mismatched types.")
                    }
                };

                let mvp = match uniforms["mvpMatrix"] {
                    ShaderType::Mat4(v) => v,
                    _ => panic!(),
                };

                attributes.insert("rangle_Position", ShaderType::Vec4(mvp * Vec4::from((position, 1.0))));

                attributes
            },
            |attributes, _uniforms| {
                let color = match attributes["color"] {
                    ShaderType::Vec4(v) => v,
                    _ => {
                        panic!("mismatched types.")
                    }
                };

                (color * 10.0).ceil() / 10.0
            },
        );

        rangle.set_display_mode(RangleMode::Triangles);

        let mut model = Model::from_file("obj/teapot.obj")?;
        let xyz_scale  = 1.5;
        model.scale(xyz_scale, xyz_scale, xyz_scale);
        model.rotate(0.0, 0.0, 0.0);
        model.translate(0.0, 0.0, 0.0);

        let mut positions = vec![];
        let vertices = model.get_vertex_buffer();
        for &vertex in &vertices {
            positions.push(ShaderType::Vec3(vertex));
        }

        shader.borrow_mut().add_attribute("position", positions)?;

        let mut colors = vec![];
        let mut v0;
        let mut v1;
        let mut v2;
        let mut normal: Vec4;
        for i in (0..vertices.len()).step_by(3) {
            v0 = vertices[i];
            v1 = vertices[i + 1];
            v2 = vertices[i + 2];

            normal = ((v1 - v0).cross(v2 - v1).abs().normalize(), 1.0).into();

            colors.push(ShaderType::Vec4(normal));
            colors.push(ShaderType::Vec4(normal));
            colors.push(ShaderType::Vec4(normal));
        }

        shader.borrow_mut().add_attribute("color", colors)?;

        let (x, y) = rangle.get_size();
        let (x, y) = (x as f32, y as f32);

        let mut m = model.compute_model_matrix();
        let v = Camera::new(
            (0.0, 0.0, 10.0).into(),
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
        )
        .compute_view_matrix();
        let p = Rangle::compute_projection_matrix(45.0_f32.to_radians(), 0.7*x / y, 0.1, 20.0);

        let mut mvp = p * v * m;
        shader
            .borrow_mut()
            .set_uniform("mvpMatrix", ShaderType::Mat4(mvp))?;

        let model = rangle.add_model(model, shader.clone());

        rangle.render_scene()?;

        let mut key;
        let mut pitch = 0.0;
        loop {
            if event::poll(Duration::from_secs(0))? {
                key = event::read()?;

                if key == Event::Key(KeyCode::Char('q').into()) {
                    break;
                }
            }
        
            now = Instant::now();

            pitch = (pitch + PI / 5.0) % (2.0 * PI);
            model.borrow_mut().rotate(0.0, pitch, 0.0);

            m = model.borrow().compute_model_matrix();
            mvp = p * v * m;
            shader
                .borrow_mut()
                .set_uniform("mvpMatrix", ShaderType::Mat4(mvp))?;

            rangle.render_scene()?;

            time_avg *= i;
            time_avg /= i+1;
            time_avg += now.elapsed() / (i+1);
            i += 1;
        }
    }

    println!("{:.3}", time_avg.as_secs_f32());

    Ok(())
}
