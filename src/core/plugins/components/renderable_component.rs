use specs::{Component, VecStorage};
use super::super::super::rendering::renderables::renderable::Renderable;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Renderable{
    pub renderable: Renderable,
}
