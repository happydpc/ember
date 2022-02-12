use crate::core::plugins::components::{CameraComponent, TransformComponent, InputComponent};
use specs::{System, ReadStorage, ReadExpect, WriteStorage, Join};
use specs::prelude::*;
use winit::event::VirtualKeyCode;
use cgmath::InnerSpace;
use std::collections::VecDeque;
use crate::core::input::input_manager::KeyInputQueue;

pub struct CameraMoveSystem;

impl<'a> System<'a> for CameraMoveSystem{
    type SystemData = (
        WriteStorage<'a, CameraComponent>,
        Read<'a, KeyInputQueue>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;
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
                    VirtualKeyCode::Q => {
                        cam.eye = cam.eye + (right * delta);
                    },
                    VirtualKeyCode::E => {
                        cam.eye = cam.eye - (right * delta);
                    },
                    _ => (),
                }
            }
        }
    }
}
