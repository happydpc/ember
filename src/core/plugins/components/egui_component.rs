// egui
use egui;

use egui::Context;
use bevy_ecs::component::Component;


// don't need to serialize this
#[derive(Component)]
pub struct EguiComponent{
    pub egui_ctx: Context,
    pub egui_painter: egui_vulkano::Painter,
}