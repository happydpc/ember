// internal imports
use crate::core::{
    managers::manager::Manager,
    plugins::{
        components::{
            renderable_component::RenderableComponent,
        },
    },
    rendering::{
        geometries::{
            geometry,
            geometry::{
                Vertex
            }
        },
        shaders::triangle::{
            vs,
            fs,
        },
    },
    scene::{
        scene::{Scene, Initialized}
    },
};

// ecs
use specs::{System, ReadStorage, ReadExpect, Read, WriteStorage, Join};
use specs::prelude::*;

// Vulkano imports
use vulkano::{
    instance::{
        Instance,
        InstanceExtensions,
    },
    device::{
        physical::{
            PhysicalDevice,
            PhysicalDeviceType,
        },
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
    swapchain,
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
            // SingleBufferDefinition,
            BuffersDefinition,
        },
        GraphicsPipeline,
        PipelineBindPoint,
    },
    sync::{
        FlushError,
        GpuFuture,
    },
    sync,
    command_buffer::{
        AutoCommandBufferBuilder,
        PrimaryAutoCommandBuffer,
        CommandBufferUsage,
        DynamicState,
        SubpassContents,
    },
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
        BufferAccess,
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

// logging
use log;


pub struct RenderManager{
    // ECS Systems
    scene_prep_system: RenderableInitializerSystem,
    command_buffer_builder_system: CommandBufferBuilderSystem,

    // Vulkan
    required_extensions: Option<InstanceExtensions>,
    device_extensions: Option<DeviceExtensions>,
    minimal_features: Option<Features>,
    optimal_features: Option<Features>,
    instance: Option<Arc<Instance>>,
    pub surface: Option<Arc<vulkano::swapchain::Surface<winit::window::Window>>>,
    pub device: Option<Arc<Device>>,
    pub queue: Option<Arc<Queue>>,
    pub swapchain: Option<Arc<Swapchain<winit::window::Window>>>,
    pub render_pass: Option<Arc<RenderPass>>,
    pub pipeline: Option<Arc<GraphicsPipeline<BuffersDefinition>>>,
    pub dynamic_state: Option<DynamicState>,
    pub framebuffers: Option<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<GpuFuture>>,
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
            .with_title("I should probably name my game.")
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        // get our physical device and queue family
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| { // filter to devices that contain desired features
                p.supported_features().is_superset_of(&optimal_features)
            })
            .filter_map(|p| { // filter queue families to ones that support graphics
                p.queue_families() // TODO : pick beter queue families since this is one single queue
                    .find(|&q| {
                        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
                    })
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| { // pick the best device based on a score we assign
                match p.properties().device_type {
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
            physical_device.properties().device_name,
            physical_device.properties().device_type,
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
        let (swapchain, images) = {
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
                // .vertex_input_single_buffer::<Vertex>()
                .vertex_input(
                    BuffersDefinition::new()
                        .vertex::<Vertex>(),
                )
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

        let framebuffers = RenderManager::window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);
        let recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // clone the surface so we can return this clone
        let return_surface = surface.clone();

        // fill options with initialized values
        self.required_extensions = Some(required_extensions);
        self.device_extensions = Some(device_extensions);
        self.minimal_features = Some(minimal_features);
        self.instance = Some(instance);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.swapchain = Some(swapchain);
        self.render_pass = Some(render_pass);
        self.pipeline = Some(pipeline);
        self.dynamic_state = Some(dynamic_state);
        self.framebuffers = Some(framebuffers);
        self.previous_frame_end = previous_frame_end;
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
            // ECS Systemes
            scene_prep_system: RenderableInitializerSystem{},
            command_buffer_builder_system: CommandBufferBuilderSystem{},

            // Vulkan
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
            framebuffers: None,
            recreate_swapchain: false,
            previous_frame_end: None,
        };
        render_sys
    }

    pub fn run(&mut self) {
        // self.window.run();
    }

    pub fn draw(&mut self, scene: &mut Scene<Initialized>){
        self.scene_prep_system.run(scene.get_world().unwrap().system_data());

        // unwrap the options we'll be using
        let mut _framebuffers = self.framebuffers.clone().unwrap();
        let mut _pipeline = self.pipeline.clone().unwrap();
        let mut _dynamic_state = self.dynamic_state.clone().unwrap();
        let mut _device = self.device.clone().unwrap();
        let mut _queue = self.queue.clone().unwrap();
        let mut _surface = self.surface.clone().unwrap();
        let mut _render_pass = self.render_pass.clone().unwrap();
        let mut _swapchain = self.swapchain.clone().unwrap();

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // if the swapchain needs to be recreated
        if self.recreate_swapchain {

            let dimensions: [u32; 2] = _surface.window().inner_size().into();
            let (new_swapchain, new_images) =
            match _swapchain.recreate().dimensions(dimensions).build() {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

            self.recreate_swapchain = false;
            _swapchain = new_swapchain;
        } // end of if on swapchain recreation

        // acquire an image from the swapchain
        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(_swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        // this is the default color of the framebuffer
        let clear_values = vec![[0.2, 0.2, 0.2, 1.0].into()];

        // create a command buffer builder
        let mut builder = AutoCommandBufferBuilder::primary(
            _device.clone(),
            _queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // create a vertex buffer
        // TODO : replace this with real geometries
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                _device.clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-0.5, -0.25, 0.0],
                    },
                    Vertex {
                        position: [0.0, 0.5, 0.0],
                    },
                    Vertex {
                        position: [0.25, -0.1, 0.0],
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        };

        let world = scene.get_world().unwrap();
        let system_data: (ReadStorage<RenderableComponent>) = world.system_data();
        let renderables = world.read_storage::<RenderableComponent>();
        // let vertex_buffers: Vec<_> = (&renderables).join();
        let mut vertex_buffers = vec!();
        let mut index_buffers = vec!();

        builder
            .begin_render_pass(
                _framebuffers[image_num].clone(),
                SubpassContents::Inline,
                clear_values,
            )
            .unwrap();

        for (renderable) in (renderables).join() {
            vertex_buffers.push(renderable.vertex_buffer.clone().unwrap().clone());
            index_buffers.push(renderable.index_buffer.clone().unwrap().clone() as Arc<BufferAccess + Send + Sync + 'static>);
            &builder.draw(
                _pipeline.clone(),
                &_dynamic_state,
                renderable.vertex_buffer.clone().unwrap().clone(),
                (),
                (),
            )
            .unwrap();
        }

        builder.end_render_pass().unwrap();

        // actually build command buffer now
        let command_buffer = builder.build().unwrap();

        // now get future state and try to draw
        // let x: u32 = _previous_frame_end.take().unwrap();
        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(_queue.clone(), _swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(_device.clone()).boxed());
            }
            Err(e) => {
                log::error!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(_device.clone()).boxed());
            }
        }

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

    pub fn prep_scene(&mut self, scene: &mut Scene<Initialized>) {
        scene.insert_resource(self.device.clone().unwrap().clone());
        scene.insert_resource(self.dynamic_state.clone().unwrap().clone());
        scene.insert_resource(self.pipeline.clone().unwrap().clone());
    }

}


pub struct RenderableInitializerSystem;


impl<'a> System<'a> for RenderableInitializerSystem{
    type SystemData = (
        ReadExpect<'a, Arc<Device>>,
        WriteStorage<'a, RenderableComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (device, mut renderable) = data;
        let device = &*device;
        for renderable in (&mut renderable).join() {
            if renderable.initialized == false{
                renderable.initialize(device.clone());
            }
        }
    }

}

pub struct CommandBufferBuilderSystem;

// impl<'a> System<'a> for CommandBufferBuilderSystem{
//     type SystemData = (
//         ReadExpect<'a, Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>>>>,
//         ReadExpect<'a, DynamicState>,
//         ReadStorage<'a, RenderableComponent>,
//         // ReadExpect<'a, AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
//     );
//
//     fn run(&mut self, data: Self::SystemData){
//         let(pipeline, dynamic_state, renderable, command_buffer) = data;
//
//     }
// }
