use specs::{Join, System, WriteStorage};
use crate::core::plugins::components::CameraComponent;

pub struct CameraInitSystem;
impl<'a> System<'a> for CameraInitSystem{
    type SystemData = WriteStorage<'a, CameraComponent>;

    fn run(&mut self, mut comps: Self::SystemData) {
        for mut cam in (&mut comps).join() {
            cam.calculate_perspective();
        }
    }
}