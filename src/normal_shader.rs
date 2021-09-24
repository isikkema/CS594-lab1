use std::{cell::RefCell, rc::Rc};

use glam::Vec4;
use rangle::{Camera, Model, Rangle, RangleError, Shader, ShaderType};

pub fn get_normal_shader(
    width: u16,
    height: u16,
    scale: (f32, f32, f32),
    rotate: (f32, f32, f32),
    translate: (f32, f32, f32),
    model: &mut Model,
) -> Result<Rc<RefCell<Shader>>, RangleError> {
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

            attributes.insert(
                "rangle_Position",
                ShaderType::Vec4(mvp * Vec4::from((position, 1.0))),
            );

            attributes
        },
        |attributes, _uniforms| {
            let color = match attributes["color"] {
                ShaderType::Vec4(v) => v,
                _ => {
                    panic!("mismatched types.")
                }
            };

            color
        },
    );

    model.scale(scale.0, scale.1, scale.2);
    model.rotate(rotate.0, rotate.1, rotate.2);
    model.translate(translate.0, translate.1, translate.2);

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

    let (x, y) = (width as f32, height as f32);

    let m = model.compute_model_matrix();
    let v = Camera::new(
        (0.0, 0.0, 10.0).into(),
        (0.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
    )
    .compute_view_matrix();
    let p = Rangle::compute_projection_matrix(45.0_f32.to_radians(), x / y, 0.1, 20.0);

    let mvp = p * v * m;
    shader
        .borrow_mut()
        .set_uniform("mvpMatrix", ShaderType::Mat4(mvp))?;

    return Ok(shader);
}
