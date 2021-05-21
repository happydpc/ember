use specs::{World, WorldExt, Builder, Component};
use glium;


pub trait Scene{
    // fn create_new() where Self: Sized;
    fn destroy(&mut self);
    fn activate(&mut self);
    fn deactivate(&mut self);
    fn update(&mut self, dt: f32);
    fn post_update(&mut self, dt: f32);
    fn draw(&mut self, frame: &mut glium::Frame);
}
