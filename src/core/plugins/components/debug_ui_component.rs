use specs::{Component, HashMapStorage};
use std::collections::VecDeque;

use egui::containers::{Window,SidePanel};


#[derive(Component)]
#[storage(HashMapStorage)]
pub struct DebugUiComponent{
    pub panel: SidePanel
}

impl DebugUiComponent{
    pub fn create() -> Self {
        DebugUiComponent{
            panel: SidePanel::left("debug_ui_panel"),
        }
    }
}