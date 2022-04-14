use specs::{System, ReadStorage, ReadExpect, Join};
use crate::core::plugins::components::{DebugUiComponent};
use egui_winit::State;
use egui_vulkano::Painter;
use egui::Context;



use log;


pub struct EguiState{
    pub ctx: Context,
    pub painter: Painter,
}

pub struct DebugUiSystem;


impl<'a> System<'a> for DebugUiSystem{
    type SystemData = (
        ReadExpect<'a, State>,
        ReadExpect<'a, EguiState>,
        ReadStorage<'a, DebugUiComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (_winit_state, egui_state, egui_comps) = data;
        let ctx = egui_state.ctx.clone();
        for _comp in (&egui_comps).join() {
            log::debug!("drawing a window");
            egui::TopBottomPanel::top("Debug")
                .show(&ctx, |ui| {
                    ui.label("Debug Panel.");
                });
            egui::Window::new("Floating Window")
                .resizable(true)
                .show(&ctx, |ui| {
                    ui.label("first label");
                    ui.label("and second");
                });
        }
    }

}