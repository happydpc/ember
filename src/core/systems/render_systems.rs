use std::sync::Arc;
use specs::{System, SystemData, ReadExpect, WriteStorage, Join};
use vulkano::device::Device;
use crate::core::plugins::components::{
    RenderableComponent,
    CameraComponent,
};

use vulkano::swapchain::Surface;
use winit::window::Window;

pub struct RenderableInitializerSystem;

impl<'a> System<'a> for RenderableInitializerSystem{
    type SystemData = (
        ReadExpect<'a, Arc<Device>>,
        WriteStorage<'a, RenderableComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (device, mut renderable) = data;
        let device = &*device;
        for renderable in (&mut renderable).join() {
            if renderable.initialized() == false{
                renderable.initialize(device.clone());
            }
        }
    }
}

pub struct CameraUpdateSystem;

impl<'a> System<'a> for CameraUpdateSystem{
    type SystemData = (
        WriteStorage<'a, CameraComponent>,
        ReadExpect<'a, Arc<Surface<Window>>>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let (mut cams, surface) = data;
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        let aspect = dimensions[0] as f32/ dimensions[1] as f32;
        for camera in (&mut cams).join(){
            camera.aspect = aspect;
            camera.calculate_view();
        }
    }
}
