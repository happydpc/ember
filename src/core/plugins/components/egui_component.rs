// egui
use egui;

use egui::Context;
use specs::{Component, HashMapStorage};


#[derive(Component)]
#[storage(HashMapStorage)]
pub struct EguiComponent{
    pub egui_ctx: Context,
    pub egui_painter: egui_vulkano::Painter,
}