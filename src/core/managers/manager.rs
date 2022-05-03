use crate::core::scene::{Scene, Active, Staged};

pub trait Manager{
    fn startup(&mut self);
    fn shutdown(&mut self);
    fn update(&mut self, scene: &mut Scene<Active>);
}
