use glium;

pub trait Renderable{
    fn initialize(&mut self, display: &glium::Display);
    fn draw(&self, frame: &mut glium::Frame);
}

pub struct RenderableData{
    // geometry: TriangleGeom,
    // vertex_buffer: Option<glium::VertexBuffer<Vector3>>,
    // index_buffer: Option<glium::IndexBuffer<u16>>,
    // normal_buffer: Option<glium::VertexBuffer<Vector3>>,
    // program: Option<glium::Program>,
}
