// egui
use egui;
use egui_winit::State;
use egui::CtxRef;
use egui_vulkano::Painter;

use specs::{Component, HashMapStorage};


#[derive(Component)]
#[storage(HashMapStorage)]
pub struct EguiComponent{
    pub egui_ctx: CtxRef,
    pub egui_painter: egui_vulkano::Painter,
}