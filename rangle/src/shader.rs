use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

use crate::error::RangleError;

#[derive(Clone, Copy, Debug)]
pub enum ShaderType {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat2(Mat2),
    Mat3(Mat3),
    Mat4(Mat4),
}

pub type ShaderTypeMap = HashMap<&'static str, ShaderType>;
type ShaderBuffer = Vec<ShaderType>;
type VertexShaderFunction = fn(ShaderTypeMap, &ShaderTypeMap) -> ShaderTypeMap;

#[derive(Clone)]
struct VertexShader {
    function: VertexShaderFunction,
    attribute_buffers: HashMap<&'static str, ShaderBuffer>,
}

impl VertexShader {
    fn new(function: VertexShaderFunction) -> Self {
        VertexShader {
            function: function,
            attribute_buffers: HashMap::new(),
        }
    }

    fn add_attribute(
        &mut self,
        name: &'static str,
        attribute_buffer: ShaderBuffer,
    ) -> Result<(), RangleError> {
        if self.attribute_buffers.get(name).is_some() {
            return Err(RangleError::DuplicateShaderAttributes);
        }

        self.attribute_buffers.insert(name, attribute_buffer);

        Ok(())
    }

    fn run(&self, index: usize, uniforms: &ShaderTypeMap) -> ShaderTypeMap {
        let mut attributes = HashMap::new();
        for (&k, v) in &self.attribute_buffers {
            attributes.insert(k, v[index]);
        }

        (self.function)(attributes, uniforms)
    }
}

type FragmentShaderFunction = fn(ShaderTypeMap, &ShaderTypeMap) -> Vec4;

#[derive(Clone)]
struct FragmentShader {
    function: FragmentShaderFunction,
}

impl FragmentShader {
    fn new(function: FragmentShaderFunction) -> Self {
        FragmentShader { function: function }
    }

    fn run(&self, attributes: ShaderTypeMap, uniforms: &ShaderTypeMap) -> Vec4 {
        (self.function)(attributes, uniforms)
    }
}

pub struct Shader {
    vertex_shader: VertexShader,
    fragment_shader: FragmentShader,
    uniforms: ShaderTypeMap,
}

impl Shader {
    pub fn new(
        vertex_function: VertexShaderFunction,
        fragment_function: FragmentShaderFunction,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Shader {
            vertex_shader: VertexShader::new(vertex_function),
            fragment_shader: FragmentShader::new(fragment_function),
            uniforms: HashMap::new(),
        }))
    }

    pub fn add_attribute(
        &mut self,
        name: &'static str,
        attribute_buffer: ShaderBuffer,
    ) -> Result<(), RangleError> {
        self.vertex_shader.add_attribute(name, attribute_buffer)?;

        Ok(())
    }

    pub fn set_uniform(
        &mut self,
        name: &'static str,
        uniform: ShaderType,
    ) -> Result<(), RangleError> {
        self.uniforms.insert(name, uniform);

        Ok(())
    }

    pub(crate) fn run_vertex(&self, index: usize) -> ShaderTypeMap {
        self.vertex_shader.run(index, &self.uniforms)
    }

    pub(crate) fn run_fragment(&self, attributes: ShaderTypeMap) -> Vec4 {
        self.fragment_shader.run(attributes, &self.uniforms)
    }
}
