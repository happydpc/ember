// import systems
pub mod systems;
pub mod math;
pub use systems::application::application_manager::Application;
pub use systems::physics::physics_system::PhysicsSystem;
pub use systems::rendering::render_system::RenderSystem;

// importing traits i guess
pub use crate::systems::core::system::System;
