use specs::{System, SystemData, ReadStorage, WriteStorage, ReadExpect, Join};
use crate::core::plugins::components::{DebugUiComponent, EguiComponent};
use egui_winit::State;
use egui_vulkano::Painter;
use egui::CtxRef;
use egui::containers::Frame;
use egui::Color32;
use egui::Window;

pub struct EguiState{
    pub ctx: CtxRef,
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
        let (winit_state, egui_state, egui_comps) = data;
        let ctx = egui_state.ctx.clone();
        for comp in (&egui_comps).join() {
            egui::Window::new("window")
                .resizable(true)
                .frame(Frame::none().fill(Color32::DARK_RED)).show(&ctx, |ui| {
                ui.label("Debug Panel!");
            });
        }
    }

}

//
//impl<'a> System<'a> for RenderableInitializerSystem{
//    type SystemData = (
//        ReadExpect<'a, Arc<Device>>,
//       WriteStorage<'a, RenderableComponent>,
//    );
//
//    fn run(&mut self, data: Self::SystemData) {
//
//        let (device, mut renderable) = data;
//        let device = &*device;
//        for renderable in (&mut renderable).join() {
//            if renderable.initialized() == false{
//                renderable.initialize(device.clone());
//            }
//        }
//    }
//
//}
//