// egui
use egui;

use egui::CtxRef;


use specs::{Component, HashMapStorage};


#[derive(Component)]
#[storage(HashMapStorage)]
pub struct EguiComponent{
    pub egui_ctx: CtxRef,
    pub egui_painter: egui_vulkano::Painter,
}