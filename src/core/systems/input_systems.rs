use crate::core::plugins::components::{CameraComponent};
use specs::{System, WriteStorage};
use specs::prelude::*;
use winit::event::VirtualKeyCode;
use cgmath::InnerSpace;

use crate::core::managers::input_manager::KeyInputQueue;

pub struct CameraMoveSystem;

impl<'a> System<'a> for CameraMoveSystem{
    type SystemData = (
        WriteStorage<'a, CameraComponent>,
        Read<'a, KeyInputQueue>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut cams, _input) = data;
        let mut input = _input.clone();
        for cam in (&mut cams).join() {
            let mut forward = (cam.eye - cam.look_at).normalize();
            forward.z = 0.0;
            let mut right = forward.cross(cam.up).normalize();
            right.z = 0.0;
            let keys = input.drain(..);
            let delta = 0.2;
            for key in keys{
                match key {
                    VirtualKeyCode::W => {
                        cam.eye = cam.eye - (forward * delta);
                        cam.look_at = cam.look_at - (forward * delta);
                    },
                    VirtualKeyCode::A => {
                        cam.eye = cam.eye + (right * delta);
                        cam.look_at = cam.look_at + (right * delta);
                    },
                    VirtualKeyCode::S => {
                        cam.eye = cam.eye + (forward * delta);
                        cam.look_at = cam.look_at + (forward * delta);
                    },
                    VirtualKeyCode::D => {
                        cam.eye = cam.eye - (right * delta);
                        cam.look_at = cam.look_at - (right * delta);
                    },
                    VirtualKeyCode::E => {
                        cam.eye = cam.eye + (right * delta);
                    },
                    VirtualKeyCode::Q => {
                        cam.eye = cam.eye - (right * delta);
                    },
                    VirtualKeyCode::F => {
                        let dx = cam.up * delta;
                        cam.eye = cam.eye + dx;
                        cam.look_at = cam.look_at + dx;
                    },
                    VirtualKeyCode::R => {
                        let dx = cam.up * delta;
                        cam.eye = cam.eye - dx;
                        cam.look_at = cam.look_at - dx;
                    },
                    _ => (),
                }
            }
        }
    }
}
