use specs::{Join, WriteStorage, System, SystemData, ReadExpect, WriteExpect};
use crate::core::plugins::components::{CameraComponent};
use vulkano::swapchain::Surface;
use winit::window::Window;
use std::sync::Arc;

use cgmath::Matrix4;