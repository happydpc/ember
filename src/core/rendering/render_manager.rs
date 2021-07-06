use crate::core::{
    managers::manager::Manager,
    rendering::{
        geometries::{
            geometry::{
                Vertex
            }
        },
        shaders::triangle::{
            vs,
            fs,
        },
    },
};
// use crate::core::rendering::window::Window;
// eventually abstract this out or use an enum to decide which window to use
// use crate::core::rendering::win_64_window::Win64Window;

use specs::System;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

// Vulkano imports
use vulkano::{
    instance::{
        Instance,
        InstanceExtensions,
        PhysicalDevice,
        PhysicalDeviceType,
    },
    device::{
        Device,
        DeviceExtensions,
        Features,
        Queue,
    },
    swapchain::{
        AcquireError,
        Swapchain,
        SwapchainCreationError,
    },
    image::{
        view::{
            ImageView,
        },
        ImageUsage,
        SwapchainImage,
    },
    render_pass::{
        Framebuffer,
        FramebufferAbstract,
        RenderPass,
        Subpass,
    },
    pipeline::{
        viewport::{
            Viewport,
        },
        vertex::{
            SingleBufferDefinition,
        },
        GraphicsPipeline,
    },
    sync::{
        FlushError,
        GpuFuture,
    },
    sync,
    command_buffer::{
        DynamicState,
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
use log;


pub struct RenderManager{
    required_extensions: Option<InstanceExtensions>,
    device_extensions: Option<DeviceExtensions>,
    minimal_features: Option<Features>,
    optimal_features: Option<Features>,
    instance: Option<Arc<Instance>>,
    surface: Option<Arc<vulkano::swapchain::Surface<winit::window::Window>>>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    swapchain: Option<Arc<Swapchain<winit::window::Window>>>,
    render_pass: Option<Arc<RenderPass>>,
    pipeline: Option<Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>>>>,
    dynamic_state: Option<DynamicState>,
    frame_buffers: Option<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl RenderManager{
    pub fn startup(&mut self) -> (EventLoop<()>, Arc<vulkano::swapchain::Surface<winit::window::Window>>){
        log::info!("Starting RenderManager...");
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

        // logging the physical device
        log::info!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name.as_ref().unwrap(),
            physical_device.properties().device_type.unwrap(),
        );

        // now create logical device and queues
        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &DeviceExtensions::required_extensions(physical_device.clone()).union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();

        // get queue
        let queue = queues.next().unwrap();

        // get swapchain, images
        let (mut swapchain, images) = {
            let caps = surface.capabilities(physical_device).unwrap();
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            let dimensions: [u32; 2] = surface.window().inner_size().into();

            // Please take a look at the docs for the meaning of the parameters we didn't mention.
            Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count)
                .format(format)
                .dimensions(dimensions)
                .usage(ImageUsage::color_attachment())
                .sharing_mode(&queue)
                .composite_alpha(composite_alpha)
                .build()
                .unwrap()
        };

        // compile our shaders
        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();

        // create our render pass
        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );

        // create our pipeline. like an opengl program but more specific
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .unwrap(),
        );

        // create dynamic state for resizing viewport
        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        let mut framebuffers = RenderManager::window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);
        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

        // clone the surface so we can return this clone
        let return_surface = surface.clone();

        // fill options with initialized values
        if self.required_extensions.is_none(){
            self.required_extensions = Some(required_extensions);
        }
        if self.device_extensions.is_none(){
            self.device_extensions = Some(device_extensions);
        }
        if self.minimal_features.is_none(){
            self.minimal_features = Some(minimal_features);
        }
        if self.instance.is_none(){
            self.instance = Some(instance);
        }
        if self.surface.is_none(){
            self.surface = Some(surface);
        }
        if self.device.is_none(){
            self.device = Some(device);
        }
        if self.queue.is_none(){
            self.queue = Some(queue);
        }
        if self.swapchain.is_none(){
            self.swapchain = Some(swapchain);
        }
        if self.render_pass.is_none(){
            self.render_pass = Some(render_pass);
        }
        if self.pipeline.is_none(){
            self.pipeline = Some(pipeline);
        }
        if self.dynamic_state.is_none(){
            self.dynamic_state = Some(dynamic_state);
        }
        if self.frame_buffers.is_none(){
            self.frame_buffers = Some(framebuffers);
        }
        if self.previous_frame_end.is_none(){
            self.previous_frame_end = previous_frame_end;
        }
        self.recreate_swapchain = false;


        (event_loop, return_surface)
    }
    pub fn shutdown(&mut self){
        log::info!("Shutting down render manager...");
    }
    pub fn update(&mut self){
    }

    pub fn create_new() -> Self {
        log::info!("Creating RenderManager...");

        // initialize our render system with all of the required vulkan components
        let render_sys = RenderManager{
            required_extensions: None,
            device_extensions: None,
            minimal_features: None,
            optimal_features: None,
            instance: None,
            surface: None,
            device: None,
            queue: None,
            swapchain: None,
            render_pass: None,
            pipeline: None,
            dynamic_state: None,
            frame_buffers: None,
            recreate_swapchain: false,
            previous_frame_end: None,
        };
        render_sys
    }

    pub fn run(&mut self) {
        // self.window.run();
    }

    /// This method is called once during initialization, then again whenever the window is resized
    fn window_size_dependent_setup(
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<RenderPass>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        images
            .iter()
            .map(|image| {
                let view = ImageView::new(image.clone()).unwrap();
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(view)
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>()
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
