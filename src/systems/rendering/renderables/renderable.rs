use glium;

pub trait Renderable{
    fn initialize(&mut self, display: &glium::Display);
    fn draw(&self, frame: &mut glium::Frame);
}
