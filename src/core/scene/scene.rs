use specs::{World, WorldExt, Builder, Component};

pub struct Scene{
    pub world: World,
}

impl Scene{
    pub fn initialize(&mut self) {
        let world = World::new();
        self.world = world;
    }
}
