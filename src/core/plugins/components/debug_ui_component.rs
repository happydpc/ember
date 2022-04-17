use specs::{Component, HashMapStorage};


use egui::containers::{SidePanel};


#[derive(Component)]
#[storage(HashMapStorage)]
pub struct DebugUiComponent{
    pub show_profiler: bool
}

impl DebugUiComponent{
    pub fn create() -> Self {
        DebugUiComponent{
            show_profiler: false,
        }
    }
}