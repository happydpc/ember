use specs::{System, SystemData, ReadStorage, WriteStorage, ReadExpect};
use crate::core::plugins::components::{DebugUiComponent, EguiComponent};
use egui_winit::State;


pub struct DebugUiSystem;


impl<'a> System<'a> for DebugUiSystem{
    type SystemData = (
        ReadExpect<'a, State>,
        ReadStorage<'a, DebugUiComponent>,
        WriteStorage<'a, EguiComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (state, ui_comp, egui_comp) = data;
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