// internal imports
use crate::core::{
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
        triangle_draw::TriangleDrawSystem,
        frame_handler::FrameSystem,
        SceneState,
    },
    scene::{
        scene::{Scene, Initialized},
    },
    systems::{
        ui_systems::EguiState,
    }
};

// ecs
use specs::{Join};
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
            QueueFamily,
        },
        Device,
        DeviceExtensions,
        Features,
        Queue,
        QueuesIter
    },
    swapchain::{
        AcquireError,
        Swapchain,
        SwapchainCreationError,
        SwapchainAcquireFuture,
    },
    swapchain,
    image::{
        view::{
            ImageView,
        },
        ImageUsage,
        SwapchainImage,
        ImageAccess,
        AttachmentImage
    },
    render_pass::{
        Framebuffer,
        RenderPass,
        Subpass,
    },
    pipeline::{
        graphics::{
            vertex_input::BuffersDefinition,
            input_assembly::InputAssemblyState,
            viewport::{Viewport, ViewportState},
            depth_stencil::DepthStencilState,
        },
        GraphicsPipeline,
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
        SubpassContents,
        synced::SyncCommandBufferBuilder,
        CommandBufferLevel,
        pool::UnsafeCommandPool,
        SecondaryCommandBuffer,
    },
    buffer::{
        TypedBufferAccess,
    },
    format::Format,
    Version,
};

// vulkano_win imports
use vulkano_win::VkSurfaceBuild;

// winit imports
use winit::{
    event_loop::{
        EventLoop
    },
    window::{
        Window,
        WindowBuilder
    },
};

// egui
use egui;

use egui::CtxRef;


// std imports
use std::sync::Arc;

// math
use cgmath::Matrix4;


// logging
use log;

pub type Aspect = [u32; 2];
pub type SwapchainImageNum = usize;
pub type TriangleSecondaryBuffers = Vec<Box<dyn SecondaryCommandBuffer>>;
pub type LightingSecondaryBuffers = Vec<Box<dyn SecondaryCommandBuffer>>;
pub type DirectionalLightingPipelne = GraphicsPipeline;
pub type AmbientLightingPipeline = GraphicsPipeline;
pub type PointLightingPipeline = GraphicsPipeline;

pub struct RenderManager{
    // Vulkan
    required_extensions: Option<InstanceExtensions>,
    device_extensions: Option<DeviceExtensions>,
    minimal_features: Option<Features>,
    optimal_features: Option<Features>,
    instance: Option<Arc<Instance>>,
    pub frame_system: Option<Arc<FrameSystem>>,
    pub triangle_drawer: Option<Arc<TriangleDrawSystem>>,
    pub surface: Option<Arc<vulkano::swapchain::Surface<winit::window::Window>>>,
    pub device: Option<Arc<Device>>,
    pub queue: Option<Arc<Queue>>,
    pub swapchain: Option<Arc<Swapchain<winit::window::Window>>>,
    pub render_pass: Option<Arc<RenderPass>>,
    pub pipeline: Option<Arc<GraphicsPipeline>>,
    pub framebuffers: Option<Vec<Arc<Framebuffer>>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub viewport: Option<Viewport>,
    pub images: Option<Vec<Arc<ImageView<SwapchainImage<winit::window::Window>>>>>,
    pub scene_state: Option<Arc<SceneState>>,
}

impl RenderManager{
    pub fn startup(&mut self) -> (EventLoop<()>, Arc<vulkano::swapchain::Surface<winit::window::Window>>){
        log::info!("Starting RenderManager...");

        // get extensions
        let (required_extensions, device_extensions) = RenderManager::get_required_extensions();

        // create an instance of vulkan with the required extensions
        let instance = Instance::new(None, Version::V1_1, &required_extensions, None).unwrap();

        // create event_loop and surface
        let (event_loop, surface) = RenderManager::create_event_loop_and_surface(instance.clone());

        // get our physical device and queue family
        let (physical_device, queue_family) = RenderManager::get_physical_device_and_queue_family(
            &instance,
            device_extensions.clone(),
            surface.clone()
        );

        // logging the physical device
        log::info!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        // now create the logical device and queues
        let (device, mut queues) = RenderManager::get_logical_device_and_queues(
            physical_device,
            &device_extensions,
            queue_family
        );

        // get queue
        let queue = queues.next().unwrap();

        // create swapchain, images
        let (swapchain, images) = RenderManager::create_swapchain_and_images(
            physical_device,
            surface.clone(),
            device.clone(),
            queue.clone()
        );

        // compile our shaders
        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        // create our render pass
        let _render_pass = vulkano::ordered_passes_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16_UNORM,
                        samples: 1,
                    }
                },
                passes: [
                    { color: [color], depth_stencil: {depth}, input: [] },
                    { color: [color], depth_stencil: {depth}, input: [] }
                ]
            )
            .unwrap();

        let render_pass = vulkano::ordered_passes_renderpass!(device.clone(),
                attachments: {
                    // The image that will contain the final rendering (in this example the swapchain
                    // image, but it could be another image).
                    final_color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    // Will be bound to `self.diffuse_buffer`.
                    diffuse: {
                        load: Clear,
                        store: DontCare,
                        // format: Format::A2R10G10B10_UNORM_PACK32,
                        format: Format::A2B10G10R10_UNORM_PACK32,
                        samples: 1,
                    },
                    // Will be bound to `self.normals_buffer`.
                    normals: {
                        load: Clear,
                        store: DontCare,
                        format: Format::R16G16B16A16_SFLOAT,
                        samples: 1,
                    },
                    // Will be bound to `self.depth_buffer`.
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16_UNORM,
                        samples: 1,
                    }
                },
                passes: [
                    // Write to the diffuse, normals and depth attachments.
                    {
                        color: [diffuse, normals],
                        depth_stencil: {depth},
                        input: []
                    },
                    // Apply lighting by reading these three attachments and writing to `final_color`.
                    {
                        color: [final_color],
                        depth_stencil: {},
                        input: [diffuse, normals, depth]
                    }
                ]
            )
            .unwrap();
        
        // create our pipeline. like an opengl program but more specific
        let pipeline = GraphicsPipeline::start()
                // We need to indicate the layout of the vertices.
                .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
                // A Vulkan shader can in theory contain multiple entry points, so we have to specify
                // which one.
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                // The content of the vertex buffer describes a list of triangles.
                .input_assembly_state(InputAssemblyState::new())
                // Use a resizable viewport set to draw over the entire window
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                // See `vertex_shader`.
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .depth_stencil_state(DepthStencilState::simple_depth_test())
                // We have to indicate which subpass of which render pass this pipeline is going to be used
                // in. The pipeline will only be usable from this particular subpass.
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
                .build(device.clone())
                .unwrap();

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let framebuffers = self.window_size_dependent_setup(&images, render_pass.clone(), &mut viewport, device.clone());
        let _recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // create our frame system
        let frame_system = FrameSystem::new(queue.clone(), swapchain.format());

        // create our triangle draw system
        let triangle_draw_system = TriangleDrawSystem::new(queue.clone(), frame_system.deferred_subpass());

        // store those two
        self.frame_system = Some(Arc::new(frame_system));
        self.triangle_drawer = Some(Arc::new(triangle_draw_system));

        // store swapchain images?
        let images = images
            .into_iter()
            .map(|image| ImageView::new(image.clone()).unwrap())
            .collect::<Vec<_>>();

        // clone the surface so we can return this clone
        let return_surface = surface.clone();

        // TODO : Somehow make this aware of when scenes are initialized and do this there instead.
        let mut scene_state = SceneState::new();
        scene_state.initialize(swapchain.clone(), device.clone());

        // fill options with initialized values
        self.required_extensions = Some(required_extensions);
        self.device_extensions = Some(device_extensions);
        self.instance = Some(instance);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.swapchain = Some(swapchain);
        self.render_pass = Some(render_pass);
        self.pipeline = Some(pipeline);
        self.framebuffers = Some(framebuffers);
        self.previous_frame_end = previous_frame_end;
        self.recreate_swapchain = false;
        self.viewport = Some(viewport);
        self.images = Some(images);
        self.scene_state = Some(Arc::new(scene_state));

        (event_loop, return_surface)
    }

    // shut down render manager
    pub fn shutdown(&mut self){
        log::info!("Shutting down render manager...");
    }

    // update render manager
    pub fn update(&mut self, _scene: &mut Scene<Initialized>){
    }

    // create a new render manager with uninitialized values
    pub fn create_new() -> Self {
        log::info!("Creating RenderManager...");

        // initialize our render system with all of the required vulkan components
        let render_sys = RenderManager{
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
            framebuffers: None,
            recreate_swapchain: false,
            previous_frame_end: None,
            viewport: None,
            images: None,
            scene_state: None,

            //deffered
            frame_system: None,
            triangle_drawer: None,
        };
        render_sys
    }

    // run the render manager
    pub fn run(&mut self) {
        // self.window.run();
    }

    pub fn draw(
        &mut self,
        scene: &mut Scene<Initialized>
    ){
        log::debug!("Entering render manager draw");
        // create primary command buffer builder
        let mut command_buffer_builder = self.get_auto_command_buffer_builder();

        // get swapchain image num and future
        let (image_num, acquire_future) = self.prep_scene_and_swapchain(scene);

        // begin main render pass
        log::debug!("Entering main render pass");
        //let clear_values = vec![[0.25, 0.2, 0.2, 1.0].into(), 1f32.into()];
        let clear_values = vec![
            [0.25, 0.2, 0.2, 1.0].into(),
            [0.0, 0.0, 0.0, 0.0].into(),
            [0.0, 0.0, 0.0, 0.0].into(),
            1.0f32.into(),
        ];
        command_buffer_builder
            .begin_render_pass(
                self.framebuffers()[image_num].clone(),
                SubpassContents::SecondaryCommandBuffers,
                clear_values,
            )
            .unwrap()
            .set_viewport(0, [self.viewport.clone().unwrap()])
            .bind_pipeline_graphics(self.pipeline());

        // insert stuff into scene that systems will need
        log::debug!("Inserting resources into scene.");
        scene.insert_resource(image_num);
        self.insert_render_data_into_scene(scene); // inserts vulkan resources into scene
        let secondary_buffer_vec: TriangleSecondaryBuffers = Vec::new(); 
        scene.insert_resource(secondary_buffer_vec);
        let lighting_buffer_vec: LightingSecondaryBuffers = Vec::new();
        scene.insert_resource(lighting_buffer_vec);
        log::debug!("Done inserting resources");

        // start egui frame
        {
            let world = scene.get_world().unwrap();
            let mut state = world.write_resource::<EguiState>();
            let mut egui_winit = world.write_resource::<egui_winit::State>();
            state.ctx.begin_frame(egui_winit.take_egui_input(self.surface().window()));
        }

        // run all systems
        log::debug!("About to run render dispatch...");
        scene.run_render_dispatch();
        log::debug!("Done with render dispatch...");

        // get and submit secondary command buffers
        {
            let world = scene.get_world().unwrap();
            let mut secondary_buffers = world.write_resource::<TriangleSecondaryBuffers>();
            // submit secondary buffers
            for buff in secondary_buffers.drain(..){
                log::debug!("Executing secondary buffer");
                command_buffer_builder.execute_commands(buff);
            }
            command_buffer_builder.next_subpass(SubpassContents::SecondaryCommandBuffers).expect("Couldn't step to deferred subpass.");
            let mut lighting_secondary_buffers = world.write_resource::<LightingSecondaryBuffers>();
            for buff in lighting_secondary_buffers.drain(..){
                log::debug!("Submitting lighting command!");
                command_buffer_builder.execute_commands(buff);
            }

        }

        // get egui shapes from world
        log::debug!("Getting egui shapes");
        let clipped_shapes = {
            let world = scene.get_world().unwrap();
            let mut state = world.write_resource::<EguiState>();
            let mut egui_winit = world.write_resource::<egui_winit::State>();
            state.ctx.begin_frame(egui_winit.take_egui_input(self.surface().window()));
            let (egui_output, clipped_shapes) = state.ctx.end_frame();
            egui_winit.handle_output(self.surface().window(), &state.ctx, egui_output);
            clipped_shapes
        };

        // send to egui
        // Automatically start the next render subpass and draw the gui
        log::debug!("submitting egui shapes to painter");
        {
            let surface = self.surface();
            let size = surface.window().inner_size();
            let sf: f32 = surface.window().scale_factor() as f32;
            let world = scene.get_world().unwrap();
            let mut state = world.write_resource::<EguiState>();
            let ctx = state.ctx.clone();
            // ctx.set_pixels_per_point(1.0);
            command_buffer_builder.set_viewport(0, [self.viewport.clone().unwrap()]);
            state.painter
                .draw(
                    &mut command_buffer_builder,
                    [(size.width as f32) / sf, (size.height as f32) / sf],
                    &ctx,
                    clipped_shapes,
                )
                .unwrap();
        }

        // end egui pass
        log::debug!("ending egui pass");
        command_buffer_builder.end_render_pass().unwrap();

        // build command buffer
        log::debug!("Building command buffer");
        let command_buffer = command_buffer_builder.build().unwrap();

        // submit and render
        log::debug!("Submitting");
        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue(), self.swapchain(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device().clone()).boxed());
            }
            Err(e) => {
                log::error!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device().clone()).boxed());
            }
        }
    }

    // render steps
    fn prep_scene_and_swapchain(
        &mut self,
        _scene: &mut Scene<Initialized>
    )->(usize, SwapchainAcquireFuture<winit::window::Window>)
    {
        // prep scene by inserting device and other operations
        // self.scene_prep_system.run(scene.get_world().unwrap().system_data()); // initializes renderables

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // acquire an image from the swapchain
        let (image_num, suboptimal, acquire_future) = self.acquire_swapchain_image();

        if suboptimal {
            self.recreate_swapchain()
        }
        (image_num, acquire_future)
    }

    pub fn get_auto_command_buffer_builder(&self)->AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>{
        // create a command buffer builder
        let builder = AutoCommandBufferBuilder::primary(
            self.device(),
            self.queue().family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        builder
    }

    /// This method is called once during initialization, then again whenever the window is resized
    fn window_size_dependent_setup(
        &mut self,
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<RenderPass>,
        viewport: &mut Viewport,
        device: Arc<Device>,
    ) -> Vec<Arc<Framebuffer>> {
        let dimensions = images[0].dimensions().width_height();
        viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
        self.viewport = Some(viewport.clone());

        let _depth_buffer = ImageView::new(
            AttachmentImage::transient(device.clone(), dimensions, Format::D16_UNORM).unwrap(),
        )
        .unwrap();

    
        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new(image.clone()).unwrap();
                // Framebuffer::start(render_pass.clone())
                //     .add(view)
                //     .unwrap()
                //     .add(depth_buffer.clone())
                //     .unwrap()
                //     .build()
                //     .unwrap()
                // TODO: use shortcut provided in vulkano 0.6
                let atch_usage = ImageUsage {
                    transient_attachment: true,
                    input_attachment: true,
                    ..ImageUsage::none()
                };

                // Note that we create "transient" images here. This means that the content of the
                // image is only defined when within a render pass. In other words you can draw to
                // them in a subpass then read them in another subpass, but as soon as you leave the
                // render pass their content becomes undefined.
                let diffuse_buffer = ImageView::new(
                    AttachmentImage::with_usage(
                        device.clone(),
                        dimensions,
                        Format::A2B10G10R10_UNORM_PACK32,
                        atch_usage,
                    )
                    .unwrap(),
                )
                .unwrap();
                let normals_buffer = ImageView::new(
                    AttachmentImage::with_usage(
                        device.clone(),
                        dimensions,
                        Format::R16G16B16A16_SFLOAT,
                        atch_usage,
                    )
                    .unwrap(),
                )
                .unwrap();
                let depth_buffer = ImageView::new(
                    AttachmentImage::with_usage(
                        device.clone(),
                        dimensions,
                        Format::D16_UNORM,
                        atch_usage,
                    )
                    .unwrap(),
                )
                .unwrap();

                // Build the framebuffer. The image must be attached in the same order as they were defined
                // with the `ordered_passes_renderpass!` macro.
                Framebuffer::start(render_pass.clone())
                    .add(view)
                    .unwrap()
                    .add(diffuse_buffer.clone())
                    .unwrap()
                    .add(normals_buffer.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap()
            })
            .collect::<Vec<_>>();
        framebuffers
    }

    // insert required render data into scene so systems can run
    pub fn insert_render_data_into_scene(&mut self, scene: &mut Scene<Initialized>) {
        let camera_state: [Matrix4<f32>; 2] = [Matrix4::from_scale(1.0), Matrix4::from_scale(1.0)];
        // insert resources. some of these should eventually be submitted more often than othrs
        scene.insert_resource(self.device());
        scene.insert_resource(self.pipeline());
        scene.insert_resource(self.surface());
        scene.insert_resource(self.queue());
        scene.insert_resource(self.viewport());
        scene.insert_resource(self.render_pass());
        scene.insert_resource(self.frame_system().normal_buffer());
        scene.insert_resource(self.frame_system().diffuse_buffer());
        // scene.insert_resource(self.frame_system().directional_lighting_pipeline());
        // scene.insert_resource(self.frame_system().ambient_lighting_pipeline());
        // scene.insert_resource(self.frame_system().point_lighting_pipeline());
        scene.insert_resource(camera_state);
        scene.insert_resource(self.scene_state());
    }

    // returns the required winit extensions and the required extensions of my choosing
    pub fn get_required_extensions() -> (InstanceExtensions, DeviceExtensions) {
        // what extensions do we need to have in vulkan to draw a window
        let required_extensions = vulkano_win::required_extensions();

        // choose the logical device extensions we're going to use
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        (required_extensions, device_extensions)
    }

    // creates a surface and ties it to the event loop
    pub fn create_event_loop_and_surface(instance: Arc<Instance>) -> (EventLoop<()>, Arc<vulkano::swapchain::Surface<winit::window::Window>>) {
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_title("Ember")
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();
        (event_loop, surface)
    }

    // gets physical GPU and queues
    pub fn get_physical_device_and_queue_family(
        instance: &Arc<Instance>,
        device_extensions: DeviceExtensions,
        surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>
    ) -> (PhysicalDevice, QueueFamily) {
        // get our physical device and queue family
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| { // filter to devices that contain desired features
                p.supported_extensions().is_superset_of(&device_extensions)
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

            (physical_device, queue_family)
    }

    // create logical device and queues. Currently a very thin pass-through
    // but it's here in case i ever want to extend this
    pub fn get_logical_device_and_queues(
        physical_device: PhysicalDevice,
        device_extensions: &DeviceExtensions,
        queue_family: QueueFamily,
    ) -> (Arc<Device>, QueuesIter){
        // now create logical device and queues
        let (device, queues) = Device::new(
            physical_device,
            physical_device.supported_features(),
            &physical_device
                .required_extensions()
                .union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        ).unwrap();

        (device, queues)
    }

    // Create swapchain and images
    pub fn create_swapchain_and_images(
        physical_device: PhysicalDevice,
        surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>,
        device: Arc<Device>,
        queue: Arc<Queue>
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let caps = surface.capabilities(physical_device).unwrap();
        let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions: [u32; 2] = surface.window().inner_size().into();

        Swapchain::start(device.clone(), surface.clone())
            .num_images(caps.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .composite_alpha(composite_alpha)
            .build()
            .unwrap()
    }

    // if the swapchain needs to be recreated
    pub fn recreate_swapchain(&mut self){
        log::debug!("Recreating swapchain...");
        let dimensions: [u32; 2] = self.surface().window().inner_size().into();
        let (new_swapchain, new_images) =
        match self.swapchain()
            .recreate()
            .dimensions(dimensions)
            .build() {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.recreate_swapchain = false;
        self.framebuffers = Some(
            self.window_size_dependent_setup(
                &new_images,
                self.render_pass(),
                &mut self.viewport(),
                self.device()
            )
        );

        // convert images to image views
        let new_images = new_images
            .into_iter()
            .map(|image| ImageView::new(image.clone()).unwrap())
            .collect::<Vec<_>>();

        self.swapchain = Some(new_swapchain);
        self.images = Some(new_images);
    } // end of if on swapchain recreation

    // acquires the next swapchain image
    pub fn acquire_swapchain_image(&mut self) -> (usize, bool, SwapchainAcquireFuture<Window>) {
        match swapchain::acquire_next_image(self.swapchain(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.recreate_swapchain();
                self.acquire_swapchain_image()
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        }
    }

    // create an egui painter
    pub fn initialize_egui(&self) -> (CtxRef, egui_vulkano::Painter){
        let egui_ctx = egui::CtxRef::default();
        let egui_painter = egui_vulkano::Painter::new(
            self.device(),
            self.queue(),
            Subpass::from(self.render_pass(), 1).unwrap(),
        )
        .unwrap();

        (egui_ctx, egui_painter)
    }

    pub fn create_egui_winit_state(&self) -> egui_winit::State{
        let surface = self.surface();
        let window = surface.window();
        egui_winit::State::new(window)
    }

    // getters
    pub fn framebuffers(&self) -> Vec<Arc<Framebuffer>> {
        self.framebuffers.clone().unwrap().clone()
    }

    pub fn pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipeline.clone().unwrap().clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone().unwrap().clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone().unwrap().clone()
    }

    pub fn surface(&self) -> Arc<vulkano::swapchain::Surface<winit::window::Window>> {
        self.surface.clone().unwrap().clone()
    }

    pub fn render_pass(&self) -> Arc<RenderPass> {
        self.render_pass.clone().unwrap().clone()
    }

    pub fn swapchain(&self) -> Arc<Swapchain<winit::window::Window>> {
        self.swapchain.clone().unwrap().clone()
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport.clone().unwrap()
    }

    pub fn frame_system(&self) -> Arc<FrameSystem> {
        self.frame_system.clone().unwrap().clone()
    }

    pub fn triangle_drawer(&self) -> Arc<TriangleDrawSystem> {
        self.triangle_drawer.clone().unwrap().clone()
    }

    pub fn images(&self) -> Vec<Arc<ImageView<SwapchainImage<winit::window::Window>>>> {
        self.images.clone().unwrap().clone()
    }

    pub fn scene_state(&self) -> Arc<SceneState> {
        self.scene_state.clone().unwrap().clone()
    }
}


pub struct PrimitiveCommandBufferBuilderSystem;
//
// impl<'a> System<'a> for PrimitiveCommandBufferBuilderSystem{
//     type SystemData = (
//         ReadExpect<'a, Arc<GraphicsPipeline>>,
//         ReadExpect<'a, Arc<DynamicState>>,
//         ReadExpect<'a, Arc<Device>>,
//         ReadStorage<'a, RenderableComponent>,
//         // ReadExpect<'a, AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>>,
//     );

//     fn run(&mut self, data: Self::SystemData){
//         let(pipeline, dynamic_state, device, renderable) = data;
//         // let(pipeline, dynamic_state, renderable, command_buffer) = data;

//         // create a command buffer builder
//         let mut builder = AutoCommandBufferBuilder::secondary_graphics(
//             self.device(),
//             self.queue().family(),
//             CommandBufferUsage::OneTimeSubmit,
//         )
//         .unwrap();

//     }
// }
