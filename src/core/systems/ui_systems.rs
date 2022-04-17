use specs::{System, ReadStorage, ReadExpect, Join, WriteStorage};
use crate::core::plugins::components::{DebugUiComponent, CameraComponent};
use egui_winit::State;
use egui_vulkano::Painter;
use egui::Context;

use puffin_egui;

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
        WriteStorage<'a, DebugUiComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (_winit_state, egui_state, mut egui_comps) = data;
        let ctx = egui_state.ctx.clone();
        for mut comp in (&mut egui_comps).join() {
            log::debug!("drawing a window");
            let mut show_profiler = comp.show_profiler;
            egui::TopBottomPanel::top("Debug")
                .show(&ctx, |ui| {
                    ui.menu_button("Debug Menu", |ui|{
                        ui.checkbox(&mut show_profiler, "Show Profiler");
                    });
                });
            if show_profiler{
                puffin_egui::profiler_window(&ctx);
            }
            comp.show_profiler = show_profiler;
        }
    }
}

pub struct CameraUiSystem;
impl<'a> System<'a> for CameraUiSystem{
    type SystemData = (
        ReadExpect<'a, EguiState>,
        WriteStorage<'a, CameraComponent>,
    );

    fn run(&mut self, data: Self::SystemData){
        let (egui_state, mut camera_comp) = data;
        let ctx = egui_state.ctx.clone();
        for mut cam in (&mut camera_comp).join(){
            let mut fov = cam.fov;
            egui::Window::new("Camera Settings")
                .show(&ctx, |ui| {
                    ui.label("FOV");
                    ui.add(egui::Slider::new(&mut fov, 0.1..=3.0))
                });
            if cam.fov != fov {
                cam.fov = fov;
                cam.calculate_perspective();
            }
        }
    }
}