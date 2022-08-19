use crate::core::plugins::components::{CameraComponent};
use winit::event::VirtualKeyCode;

use crate::core::managers::input_manager::KeyInputQueue;
use bevy_ecs::prelude::{Query, Res, ResMut};
// pub struct CameraMoveSystem;

pub fn InputPrepSystem(
    input_queue: ResMut<KeyInputQueue>,
    modifier_state: ResMut<Option<VirtualKeyCode>>,
){

}

pub fn CameraMoveSystem(
    mut query: Query<&mut CameraComponent>,
    input_queue: Res<KeyInputQueue>
) {
    let mut input = input_queue.clone();
    for mut cam in query.iter_mut() {
        let mut forward = (cam.eye - cam.look_at).normalize();
        forward.z = 0.0;
        let mut right = forward.cross(cam.up).normalize();
        right.z = 0.0;
        let keys = input.drain(..);
        let delta = 0.2;
        for key in keys{
            match key {
                VirtualKeyCode::W => {
                    cam.eye = cam.eye - (forward.scale(delta));
                    cam.look_at = cam.look_at - (forward.scale(delta));
                },
                VirtualKeyCode::A => {
                    cam.eye = cam.eye + (right.scale(delta));
                    cam.look_at = cam.look_at + (right.scale(delta));
                },
                VirtualKeyCode::S => {
                    cam.eye = cam.eye + (forward.scale(delta));
                    cam.look_at = cam.look_at + (forward.scale(delta));
                },
                VirtualKeyCode::D => {
                    cam.eye = cam.eye - (right.scale(delta));
                    cam.look_at = cam.look_at - (right.scale(delta));
                },
                VirtualKeyCode::E => {
                    cam.eye = cam.eye + (right.scale(delta));
                },
                VirtualKeyCode::Q => {
                    cam.eye = cam.eye - (right.scale(delta));
                },
                VirtualKeyCode::F => {
                    let dx = cam.up.scale(delta);
                    cam.eye = cam.eye + dx;
                    cam.look_at = cam.look_at + dx;
                },
                VirtualKeyCode::R => {
                    let dx = cam.up.scale(delta);
                    cam.eye = cam.eye - dx;
                    cam.look_at = cam.look_at - dx;
                },
                _ => (),
            }
        }
    }
}
