use crate::core::plugins::components::{CameraComponent, TransformComponent, InputComponent};
use specs::{System, ReadStorage, ReadExpect, WriteStorage, Join};
use specs::prelude::*;
use winit::event::VirtualKeyCode;
use cgmath::InnerSpace;
use std::collections::VecDeque;


pub struct CameraMoveSystem{
    pub key_input_queue: VecDeque<VirtualKeyCode>,
}

impl<'a> System<'a> for CameraMoveSystem{
    type SystemData = (
        WriteStorage<'a, CameraComponent>,
        WriteStorage<'a, InputComponent>,
    );

    fn run(&mut self, (mut cam, mut input): Self::SystemData) {
        use specs::Join;
        for (cam, input) in (&mut cam, &mut input).join() {
            let forward = (cam.eye - cam.look_at).normalize();
            let right = forward.cross(cam.up).normalize();
            let keys = self.key_input_queue.drain(..);
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


