use crate::core::plugins::components::{CameraComponent};
use winit::event::VirtualKeyCode;

use crate::core::managers::input_manager::KeyInputQueue;
use bevy_ecs::prelude::{Query, Res, ResMut};
// pub struct CameraMoveSystem;
use ember_math::Matrix3f;
use ember_math::Matrix4f;
use ember_math::Vector3f;

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
                    // // cam.eye = cam.eye + (right.scale(delta));
                    // let mut v = cam.eye - cam.look_at;
                    // let rm = Matrix3f::from_axis_angle(Vector3f::new(0.0, 0.0, 1.0), -0.000001);
                    // let vr = rm.transform(v);
                    // let vt = vr + cam.look_at;
                    // cam.eye = vr;
                    let mut v = cam.eye - cam.look_at;
                    // let m = Matrix4f::from_translation(cam.look_at.scale(-1.0));
                    let rm = Matrix4f::from_axis_angle(cam.up.normalize(), -0.05);

                    // let translated_eye = m.transform(cam.eye.extend(1.0));
                    cam.eye = rm.transform(cam.eye.extend(1.0)).truncate();
                    // cam.eye = Matrix4f::from_translation(cam.look_at).transform(rotated_eye).truncate();
                    // let vr = rm.transform(v.extend(1.0));
                    // let vt = vr + cam.look_at;
                    // cam.eye = vr;
                },
                VirtualKeyCode::Q => {
                    // cam.eye = cam.eye - (right.scale(delta));
                    let mut v = cam.eye - cam.look_at;
                    let m = Matrix4f::from_translation(cam.look_at.scale(-1.0));
                    let rm = Matrix4f::from_axis_angle(cam.up, 0.05);

                    let translated_eye = m.transform(cam.eye.extend(1.0));
                    let rotated_eye = rm.transform(translated_eye);
                    cam.eye = Matrix4f::from_translation(cam.look_at).transform(rotated_eye).truncate();
                    // let vr = rm.transform(v.extend(1.0));
                    // let vt = vr + cam.look_at;
                    // cam.eye = vr;
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
    }
}
