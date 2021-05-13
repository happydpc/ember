use crate::math::structures::vector::Vector3;

use super::{
    super::{
        geometries::triangle::TriangleGeom,
        shaders::triangle::{
            fs::FRAGMENT_SHADER_SRC,
            vs::VERTEX_SHADER_SRC,
        },
    },
    renderable::Renderable,

};
use glium;
use glium::Surface;


pub struct Triangle{
    geometry: TriangleGeom,
    vertex_buffer: Option<glium::VertexBuffer<Vector3>>,
    index_buffer: Option<glium::IndexBuffer<u16>>,
    normal_buffer: Option<glium::VertexBuffer<Vector3>>,
    program: Option<glium::Program>,
}

impl Triangle{
    pub fn create(x: f32, y: f32, z: f32) -> Self{
        let instance = Triangle{
            geometry: TriangleGeom::create(x, y, z),
            vertex_buffer: None,
            index_buffer: None,
            normal_buffer: None,
            program: None,
        };
        instance
    }
}

impl Renderable for Triangle{
    fn initialize(&mut self, display: &glium::Display){
        // initialize the vertex buffer, index buffer, and shader program
        if self.vertex_buffer.is_none(){
            self.vertex_buffer = Some(glium::VertexBuffer::new(display, &self.geometry.vertices).unwrap());
        }
        if self.index_buffer.is_none(){
            self.index_buffer = Some(glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList,
            &self.geometry.indices).unwrap());
        }
        if self.program.is_none(){
            self.program = Some(glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap());
        }

    }
    fn draw(&self, frame: &mut glium::Frame){
        frame.draw(
            self.vertex_buffer.as_ref().unwrap(),
            self.index_buffer.as_ref().unwrap(),
            self.program.as_ref().unwrap(),
            &glium::uniforms::EmptyUniforms,
            &Default::default()
        ).unwrap();
    }
}
