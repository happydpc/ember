use specs::{Component, VecStorage};
use super::super::super::rendering::renderables::renderable::Renderable;
use std::sync::Mutex;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct RenderableComponent{
    pub renderable: Mutex<Box<Renderable>>,
}
