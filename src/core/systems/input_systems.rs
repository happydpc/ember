use crate::core::{plugins::components::{CameraComponent}, managers::input_manager::MouseState};
use winit::event::VirtualKeyCode;

use crate::core::managers::input_manager::KeyInputQueue;
use bevy_ecs::prelude::{Query, Res, ResMut};
// pub struct CameraMoveSystem;

use ember_math::{Matrix4f, Vector3f};


pub fn InputPrepSystem(
    _input_queue: ResMut<KeyInputQueue>,
    _modifier_state: ResMut<Option<VirtualKeyCode>>,
){

}

pub fn CameraMoveSystem(
    mut query: Query<&mut CameraComponent>,
    input_queue: Res<KeyInputQueue>,
    mouse_state: Res<MouseState>
) {
    let mut input = input_queue.clone();
    let mut mouse_state = mouse_state.clone();
    for mut cam in query.iter_mut() {
        let mut forward = (cam.eye - cam.look_at).normalize();
        forward.y = 0.0;
        let mut right = forward.cross(cam.up).normalize();
        right.y = 0.0;
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
                    let _v = cam.eye - cam.look_at;
                    // let rm = Matrix4f::from_axis_angle(cam.up.normalize(), -0.05);
                    let rm = Matrix4f::from_axis_angle(Vector3f::new(0.0, 1.0, 0.0), -0.05);

                    cam.eye = rm.transform(cam.eye.extend(1.0)).truncate();

                    // let up = cam.up.normalize();
                    // let right = cam.look_at.cross(cam.up).normalize();
                    // let camVec = cam.look_at - cam.eye;
                },
                VirtualKeyCode::Q => {
                    let _v = cam.eye - cam.look_at;
                    let m = Matrix4f::from_translation(cam.look_at.scale(-1.0));
                    let rm = Matrix4f::from_axis_angle(cam.up, 0.05);

                    let translated_eye = m.transform(cam.eye.extend(1.0));
                    let rotated_eye = rm.transform(translated_eye);
                    cam.eye = Matrix4f::from_translation(cam.look_at).transform(rotated_eye).truncate();
                },
                VirtualKeyCode::F => {
                    let dx = cam.up.scale(delta);
                    cam.eye = cam.eye - dx;
                    cam.look_at = cam.look_at - dx;
                },
                VirtualKeyCode::R => {
                    let dx = cam.up.scale(delta);
                    cam.eye = cam.eye + dx;
                    cam.look_at = cam.look_at + dx;
                },
                _ => (),
            }
        }

        if mouse_state.mouse_down[2] && mouse_state.mouse_delta.x != 0.0 {
            let rm = Matrix4f::from_axis_angle(Vector3f::new(0.0, 1.0, 0.0), mouse_state.mouse_delta.x * 0.05);
            cam.eye = rm.transform(cam.eye.extend(1.0)).truncate();
        }

        if mouse_state.scroll[1] != 0.0 {
            let dx = mouse_state.scroll[1] * -0.05;
            let scale = 1.0 + dx;
            cam.eye = cam.eye.scale(scale);
        }

        if mouse_state.mouse_down[1] && mouse_state.mouse_delta.y != 0.0 {
            let dx = -0.05 * mouse_state.mouse_delta.y;
            let new_focus = cam.look_at + Vector3f::new(0.0, dx, 0.0);
            cam.look_at = new_focus;
            cam.calculate_perspective();
        }
        cam.calculate_view();

    }

}
