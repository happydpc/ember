use specs::{World, WorldExt, Builder, Component};


pub struct Scene{
    pub world: World,
}

impl Scene{
    pub fn create_new() -> Self {
        Scene{
            world: World::new(),
        }
    }
}
