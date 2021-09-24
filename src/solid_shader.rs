use std::{cell::RefCell, rc::Rc};

use glam::Vec4;
use rangle::{Camera, Model, Rangle, RangleError, Shader, ShaderType};

pub fn get_solid_shader(
    width: u16,
    height: u16,
    color: (f32, f32, f32, f32),
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

    let colors = vec![ShaderType::Vec4(color.into()); vertices.len()];

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
