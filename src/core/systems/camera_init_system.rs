use bevy_ecs::prelude::{Query};

use crate::core::plugins::components::CameraComponent;

pub fn CameraInitSystem(
    mut query: Query<&mut CameraComponent>
)
{
    log::debug!("Camera init system...");
    for mut cam in query.iter_mut() {
        cam.calculate_perspective();
    }
}