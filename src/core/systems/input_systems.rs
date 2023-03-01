use crate::core::{plugins::components::{CameraComponent}, managers::input_manager::MouseState};
use winit::event::VirtualKeyCode;

use crate::core::managers::input_manager::KeyInputQueue;
use bevy_ecs::prelude::{Query, Res, ResMut};
// pub struct CameraMoveSystem;

use ember_math::{Matrix4f, Vector3f};


pub fn CameraMoveSystem(
    mut query: Query<&mut CameraComponent>,
    input_queue: Res<KeyInputQueue>,
    mouse_state: Res<MouseState>
) {
    let mut input = input_queue.queue.clone();
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
            cam.azimuth += mouse_state.mouse_delta.x * cam.orbit_speed;
            cam.update_cartesian();
        }

        if mouse_state.mouse_down[2] && mouse_state.mouse_delta.y != 0.0 {
            let mut new_declination = cam.declination - mouse_state.mouse_delta.y * cam.orbit_speed / 2.0;
            new_declination = 0.001_f32.max(new_declination).min(core::f32::consts::PI - 0.001);
            cam.declination = new_declination;
            cam.update_cartesian();
        }

        if mouse_state.scroll[1] != 0.0 {
            let dx = mouse_state.scroll[1] * -0.05 * cam.radius;
            cam.radius += dx;
            cam.update_cartesian();
        }

        // if mouse_state.mouse_down[1] {
        //     let dx = mouse_state.mouse_delta.x;
        //     let dy = mouse_state.mouse_delta.y;
        //     let look_rel_eye = cam.look_at - cam.eye;
        //     let old_azimuth = (look_rel_eye.x / look_rel_eye.z).atan();
        //     let mut old_declination = (look_rel_eye.x * look_rel_eye.x + look_rel_eye.z * look_rel_eye.z).sqrt();
        //     old_declination = old_declination / look_rel_eye.y;
        //     old_declination = old_declination.atan();

        //     let new_declination = old_declination - dy * cam.orbit_speed / 2.0;
        //     let new_azimuth = old_azimuth + dx * cam.orbit_speed;
        //     let r = look_rel_eye.magnitude();
        //     let new_look_x = r * new_declination.sin() * new_azimuth.cos();
        //     let new_look_y = r * new_declination.cos();
        //     let new_look_z = r * new_declination.sin() * new_azimuth.sin();
        //     let new_look = Vector3f::new(new_look_x, new_look_y, new_look_z) + cam.eye;
        //     cam.look_at = new_look;
        //     cam.update();
        // }

    }

}
