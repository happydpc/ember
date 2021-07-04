use crate::core::managers::manager::Manager;
// use crate::core::rendering::window::Window;
// eventually abstract this out or use an enum to decide which window to use
// use crate::core::rendering::win_64_window::Win64Window;

use specs::System;

// Vulkano imports
use vulkano::{
    instance::{
        Instance,
        InstanceExtensions,
        PhysicalDevice,
        PhysicalDeviceType
    },
    device::{
        Device,
        DeviceExtensions,
        Features
    },
    swapchain::{
        AcquireError,
        Swapchain,
        SwapchainCreationError
    },
    Version,
};

// vulkano_win imports
use vulkano_win::{
    VkSurfaceBuild,
};

// winit imports
use winit::{
    event::{
        Event,
        WindowEvent
    },
    event_loop::{
        ControlFlow,
        EventLoop
    },
    window::{
        Window,
        WindowBuilder
    },
};

// std imports
use std::sync::Arc;


pub struct RenderManager{
    required_extensions: InstanceExtensions,
    device_extensions: DeviceExtensions,
    minimal_features: Features,
    optimal_features: Features,
    instance: Arc<Instance>,
    event_loop: EventLoop,
    surface: vulkano::swapchain::Surface,
}

impl Manager for RenderManager{
    fn startup(&mut self){
        println!("Starting RenderManager...");
        // self.window.init();
    }
    fn shutdown(&mut self){
        println!("Shutting down render manager...");
    }
    fn update(&mut self){
    }
}
impl RenderManager{
    // TODO : add a parameter for window type
    pub fn create_new() -> Self{
        println!("Creating RenderManager...");

        // what extensions do we need to have in vulkan to draw a window
        let required_extensions = vulkano_win::required_extensions();

        // choose the logical device extensions we're going to use
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        // choose the minimal features we want our physical device to have
        let minimal_features = Features {
            geometry_shader: true,
            .. Features::none()
        };

        // choose the optimal features we want our device to have
        let optimal_features = vulkano::device::Features {
            geometry_shader: true,
            tessellation_shader: true,
            .. Features::none()
        };

        // create an instance of vulkan with the required extensions
        let instance = Instance::new(None, Version::V1_1, &required_extensions, None).unwrap();

        // create event_loop and surface
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        // get our physical device and queue family
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| { // filter to devices that contain desired features
                p.supported_features().superset_of(&optimal_features)
            })
            .filter_map(|p| { // filter queue families to ones that support graphics
                p.queue_families() // TODO : pick beter queue families since this is one single queue
                    .find(|&q| {
                        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
                    })
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| { // pick the best device based on a score we assign
                match p.properties().device_type.unwrap() {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                }
            })
            .unwrap();

        // initialize our render system with all of the required vulkan components
        let render_sys = RenderManager{
            required_extensions: required_extensions,
            device_extensions: device_extensions,
            minimal_features: minimal_features,
            optimal_features: optimal_features,
            instance: instance,
            event_loop: event_loop,
            surface: surface,
        };
        render_sys
    }
    pub fn run(&mut self) {
        // self.window.run();
    }
}

pub struct RenderableInitializerSystem;

impl<'a> System<'a> for RenderableInitializerSystem{
    type SystemData = ();
    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        // for renderable in

    }
}
