// internal imports
use crate::core::{
    rendering::{
        SceneState,
    },
    scene::{
        scene::{Scene, Active, Staged},
    },
    systems::{
        ui_systems::EguiState,
    },
    plugins::components::{
        CameraMatrices,
    },
};

use bevy_ecs::prelude::Resource;


use egui_vulkano::UpdateTexturesError;
// Vulkano imports
use vulkano::{
    VulkanLibrary,
    instance::{
        Instance,
        InstanceExtensions,
        InstanceCreateInfo, debug::{DebugUtilsMessenger, DebugUtilsMessengerCreateInfo, DebugUtilsMessageSeverity, DebugUtilsMessageType},
    },
    device::{
        physical::{
            PhysicalDevice,
            PhysicalDeviceType,
        },
        Device,
        DeviceExtensions,
        Queue,
        DeviceCreateInfo,
        QueueCreateInfo,
    },
    swapchain::{
        AcquireError,
        Swapchain,
        SwapchainCreationError,
        SwapchainAcquireFuture,
        SwapchainCreateInfo,
        SwapchainPresentInfo,
    },
    memory::{
        allocator::StandardMemoryAllocator,
    },
    swapchain,
    image::{
        view::{
            ImageView,
        },
        ImageUsage,
        SwapchainImage,
        AttachmentImage
    },
    pipeline::{
        GraphicsPipeline,
    },
    sync::{
        FlushError,
        GpuFuture,
    },
    format::Format,
    sync,
    command_buffer::{
        AutoCommandBufferBuilder,
        PrimaryAutoCommandBuffer,
        CommandBufferUsage,
        SubpassContents,
        SecondaryAutoCommandBuffer,
        RenderPassBeginInfo,
        allocator::StandardCommandBufferAllocator,
        SubpassContents::Inline
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
    },
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
use egui::{self, FullOutput};

use egui::Context;
use winit::event_loop::EventLoopWindowTarget;


// std imports
use std::sync::{Arc};

// math
use ember_math::Matrix4f;

// logging
use log;

#[derive(Resource)]
pub struct TriangleSecondaryBuffers{
    pub buffers: Vec<Box<SecondaryAutoCommandBuffer>>
}

#[derive(Resource)]
pub struct LightingSecondaryBuffers{
    pub buffers: Vec<Box<SecondaryAutoCommandBuffer>>
}

#[derive(Resource)]
pub struct DeviceResource(pub Arc<Device>);

#[derive(Resource)]
pub struct QueueResource(pub Arc<Queue>);

#[derive(Resource)]
pub struct SceneStateResource(pub Arc<SceneState>);

#[derive(Resource)]
pub struct SurfaceResource(pub Arc<vulkano::swapchain::Surface>);

#[derive(Resource)]
pub struct VulkanAllocators {
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl VulkanAllocators{
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    ) -> Self {
        VulkanAllocators {
            memory_allocator: memory_allocator,
            command_buffer_allocator: command_buffer_allocator,
            descriptor_set_allocator: descriptor_set_allocator,
        }
    }
}


pub struct RenderManager{
    // Vulkan
    required_extensions: InstanceExtensions,
    device_extensions: DeviceExtensions,
    instance: Arc<Instance>,
    pub surface: Arc<vulkano::swapchain::Surface>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub images: Vec<Arc<ImageView<SwapchainImage>>>,
    pub scene_state: Arc<SceneState>,
    pub allocators: VulkanAllocators,
}

impl RenderManager{
    pub fn new() -> (Self, EventLoop<()>) {
        log::info!("Starting RenderManager...");

        // get library
        let library = VulkanLibrary::new().unwrap();

        println!("List of Vulkan debugging layers available to use:");
        let layers = library.layer_properties().unwrap();
        for l in layers {
            println!("\t{}", l.name());
        }

        // NOTE: To simplify the example code we won't verify these layer(s) are actually in the layers list:
        let layers = vec!["VK_LAYER_KHRONOS_validation".to_owned()];

        // get extensions
        let (required_extensions, device_extensions) = RenderManager::get_required_extensions(&library);

        // // create an instance of vulkan with the required extensions
        // let instance = Instance::new(
        //     library,
        //     // None, Version::V1_1, &required_extensions, None
        //     InstanceCreateInfo{
        //         enabled_extensions: required_extensions,
        //         enumerate_portability: true,
        //         ..Default::default()
        //     }
        // ).unwrap();

        // Important: pass the extension(s) and layer(s) when creating the vulkano instance
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                enabled_layers: layers,
                // Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
                enumerate_portability: true,
                ..Default::default()
            },
        )
        .expect("failed to create Vulkan instance");

        let _debug_callback = unsafe {
            DebugUtilsMessenger::new(
                instance.clone(),
                DebugUtilsMessengerCreateInfo {
                    message_severity: DebugUtilsMessageSeverity {
                        error: true,
                        warning: true,
                        information: true,
                        verbose: true,
                        ..DebugUtilsMessageSeverity::empty()
                    },
                    message_type: DebugUtilsMessageType {
                        general: true,
                        validation: true,
                        performance: true,
                        ..DebugUtilsMessageType::empty()
                    },
                    ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                        let severity = if msg.severity.error {
                            "error"
                        } else if msg.severity.warning {
                            "warning"
                        } else if msg.severity.information {
                            "information"
                        } else if msg.severity.verbose {
                            "verbose"
                        } else {
                            panic!("no-impl");
                        };
    
                        let ty = if msg.ty.general {
                            "general"
                        } else if msg.ty.validation {
                            "validation"
                        } else if msg.ty.performance {
                            "performance"
                        } else {
                            panic!("no-impl");
                        };
    
                        println!(
                            "{} {} {}: {}",
                            msg.layer_prefix.unwrap_or("unknown"),
                            ty,
                            severity,
                            msg.description
                        );
                    }))
                },
            )
            .ok()
        };

        // create event_loop and surface
        let (event_loop, surface) = RenderManager::create_event_loop_and_surface(instance.clone());

        // get our physical device and queue family
        let (physical_device, queue_family_index) = RenderManager::get_physical_device_and_queue_family_index(
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
            physical_device.clone(),
            device_extensions.clone(),
            queue_family_index
        );

        // create our memory allocator which I think allocates command buffers (?). docs say it's general use 
        // https://docs.rs/vulkano/latest/vulkano/memory/allocator/type.StandardMemoryAllocator.html
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let cmd_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone()));
        let allocator_set = VulkanAllocators::new(memory_allocator, cmd_buffer_allocator, descriptor_set_allocator);

        // get queue
        let queue = queues.next().unwrap();

        // create swapchain, images
        let (swapchain, images) = RenderManager::create_swapchain_and_images(
            physical_device.clone(),
            surface.clone(),
            device.clone(),
            queue.clone()
        );

        // TODO : Somehow make this aware of when scenes are Active and do this there instead.
        let scene_state = SceneState::new(swapchain.clone(), device.clone(), memory_allocator.clone());
        scene_state.scale_scene_state_to_images(&images);

        // store swapchain images?
        let images = images
            .into_iter()
            .map(|image| ImageView::new_default(image).unwrap())
            .collect::<Vec<_>>();

        let previous_frame_end = Some(sync::now(device.clone()).boxed());
        
        // fill options with initialized values
        let render_manager = RenderManager {
            required_extensions: required_extensions,
            device_extensions: device_extensions,
            instance: instance,
            surface: surface,
            device: device,
            queue: queue,
            swapchain: swapchain,
            previous_frame_end: previous_frame_end,
            recreate_swapchain: false,
            images: images,
            scene_state: scene_state,
            allocators: allocator_set,
        };

        (render_manager, event_loop)
    }

    // shut down render manager
    pub fn shutdown(&mut self){
        log::info!("Shutting down render manager...");
    }

    // update render manager
    pub fn update(&mut self, _scene: &mut Scene<Active>){
    }

    pub fn prep_staged_scene(&mut self, scene: &mut Scene<Staged>){
        log::info!("Render Manager prepping scene...");

        scene.get_world()
            .unwrap()
            .init_resource::<SurfaceResource>();
        scene.get_world()
            .unwrap()
            .init_resource::<CameraMatrices>();
        scene.get_world()
            .unwrap()
            .init_resource::<DeviceResource>();
        scene.get_world()
            .unwrap()
            .init_resource::<QueueResource>();
        scene.get_world()
            .unwrap()
            .init_resource::<SceneStateResource>();
        scene.get_world()
            .unwrap()
            .init_resource::<VulkanAllocators>();
        // get required egui data
        let (egui_ctx, egui_painter) = self.initialize_egui();

        let egui_state = EguiState{ctx: egui_ctx, painter: egui_painter};
        let secondary_buffer_vec: TriangleSecondaryBuffers = TriangleSecondaryBuffers{buffers: Vec::new()}; 
        let lighting_buffer_vec: LightingSecondaryBuffers = LightingSecondaryBuffers{buffers: Vec::new()};
        let camera_state = CameraMatrices::default();
        
        scene.insert_resource(secondary_buffer_vec); // renderable vec to fill
        scene.insert_resource(lighting_buffer_vec);
        scene.insert_resource(egui_state);

        scene.insert_resource(camera_state);

        let device_resource = DeviceResource(self.device.clone());
        let queue_resource = QueueResource(self.queue.clone());
        let surface_resource = SurfaceResource(self.surface.clone());
        let scene_state_resource = SceneStateResource(self.scene_state.clone());

        scene.insert_resource(device_resource);
        scene.insert_resource(queue_resource);
        scene.insert_resource(surface_resource);
        scene.insert_resource(scene_state_resource);

        scene.insert_resource(self.allocators.clone());
    }

    pub fn draw(
        &mut self,
        scene: &mut Scene<Active>,
        egui_winit_state: &mut egui_winit::State,
    ){
        log::debug!("Entering draw");
        self.previous_frame_end.as_mut().unwrap().as_mut().cleanup_finished();

        // get swapchain image num and future
        // let (image_num, acquire_future) = self.prep_swapchain();
        let result = self.prep_swapchain();
        let (image_num,  acquire_future) = match result {
            Ok(r) => r,
            Err(e) => {
                log::error!("Error Acquiring Swapchain Image. Returning instead. {:?}", e);
                return
            }
        };

        // I believe this join basically forces the program to wait until the swapchain image we requested is ready
        // let future = self.previous_frame_end.take().unwrap().join(acquire_future);

        // scales framebuffer and attachments to swapchain image view of swapchain image
        // self.scene_state().scale_scene_state_to_images(self.images()[image_num as usize].clone());

        // create primary command buffer builder
        let mut command_buffer_builder = self.get_auto_command_buffer_builder();

        // start egui frame
        self.start_egui_frame(scene, egui_winit_state);

        // run all systems. This will build secondary command buffers
        log::debug!("----Render schedule-----");
        scene.run_render_schedule();

        // get egui shapes from world
        log::debug!("Getting egui shapes");
        let egui_output: FullOutput = self.get_egui_output(scene, egui_winit_state);
        let result = self.update_egui_textures(&egui_output, scene);
        let egui_texture_future = result.expect("Egui texture future not found from update_textures");

        // set clear values and begin the render pass
        self.begin_render_pass(&mut command_buffer_builder, image_num);

        // get and submit secondary command buffers to render pass
        submit_render_system_command_buffers_to_render_pass(scene, &mut command_buffer_builder);

        // add egui draws to command buffer
        self.draw_egui(scene, &mut command_buffer_builder, egui_output);

        // end pass
        log::debug!("ending render pass");
        command_buffer_builder.end_render_pass().unwrap();

        // build command buffer
        log::debug!("Building command buffer");
        // let command_buffer = command_buffer_builder.build().unwrap();
        let result = command_buffer_builder.build();
        let command_buffer = match result {
            Ok(res) => {
                log::debug!("Successfylly built command buffer");
                res
            },
            Err(err) => {
                log::debug!("something went wrong");
                panic!(err)
            }
        };

        // join futures
        log::debug!("Joining futures");
        // this puts us at the moment when the egui textures are done updating and our image is ready to be drawn
        let mut future_mut = acquire_future.join(egui_texture_future).boxed();
        if let Some(future) = self.previous_frame_end.take()  {
            future_mut = future_mut.join(future).boxed();
        }

        // submit and render
        self.submit_command_buffer_and_render(future_mut, command_buffer, image_num);

        let mut world = scene.get_world().unwrap();
        world
            .get_resource_mut::<EguiState>()
            .expect("Couldn't get egui state.")
            .painter
            .free_textures();
    }

    fn submit_command_buffer_and_render(
        &mut self,
        acquire_future: Box<dyn GpuFuture>,
        command_buffer: PrimaryAutoCommandBuffer,
        image_num: u32
    ) {
        log::debug!("Submitting gpu commands on image {:?}", image_num);
        let future = acquire_future
            .then_execute(
                self.queue.clone(),
                command_buffer
            )
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    image_num
                ),
            )
            .then_signal_fence_and_flush();
        log::debug!("Done submitting");
        match future {
            Ok(future) => {
                log::debug!("Future flushed successfuly");
                // self.previous_frame_end = Some(sync::now(self.device().clone()).boxed());
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                log::error!("Flush Error: Out of date");
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                log::error!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        }
    }

    fn draw_egui(&mut self, scene: &mut Scene<Active>, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>, egui_output: FullOutput) {
        log::debug!("Drawing egui");
        let binding = self.surface;
        let window = Arc::new(binding.object().unwrap().downcast_ref::<Window>().unwrap());
        let size = window.inner_size();
        let sf: f32 = window.scale_factor() as f32;
        let mut world = scene.get_world().unwrap();
        let ctx = world.get_resource_mut::<EguiState>().expect("Couldn't get egui state.").ctx.clone();
        world
            .get_resource_mut::<EguiState>()
            .expect("Couldn't get egui state.")
            .painter
            .draw(
                command_buffer_builder,
                [(size.width as f32) / sf, (size.height as f32) / sf],
                &ctx,
                egui_output.shapes,
                self.scene_state.viewport()
            )
            .unwrap();
    }

    fn start_egui_frame(
        &mut self,
        scene: &mut Scene<Active>,
        egui_winit_state: &mut egui_winit::State,
    ) {
        let mut world = scene.get_world().unwrap();
        let ctx = world.get_resource_mut::<EguiState>().expect("Couldn't get egui state.").ctx.clone();
        let binding = self.surface;
        let window = Arc::new(binding.object().unwrap().downcast_ref::<Window>().unwrap());
        ctx.begin_frame(egui_winit_state.take_egui_input(&window));
    }

    fn get_egui_output(
        &mut self,
        scene: &mut Scene<Active>,
        egui_winit_state: &mut egui_winit::State
    ) -> egui::FullOutput {
        let ctx = scene.get_world().unwrap().get_resource_mut::<EguiState>().expect("Couldn't get egui state").ctx.clone();

        let egui_output = ctx.end_frame();

        let platform_output = egui_output.platform_output.clone();
        let binding = self.surface;
        let window = Arc::new(binding.object().unwrap().downcast_ref::<Window>().unwrap());
        
        egui_winit_state.handle_platform_output(
            &window,
            &ctx,
            platform_output
        );
        egui_output
    }

    fn update_egui_textures(
        &mut self,
        egui_output: &FullOutput,
        scene: &mut Scene<Active>
    ) ->  Result<impl  GpuFuture, UpdateTexturesError>{
        let textures_delta = egui_output.textures_delta.clone();
        if let Some(mut egui_state) = scene.get_world().unwrap().get_resource_mut::<EguiState>() {
            egui_state.painter.update_textures(textures_delta, self.allocators.command_buffer_allocator.clone())
        } else {
            log::info!("lol so remember how I said I should add error handling? well  yeah.");
            Err(UpdateTexturesError::NoEguiState)
        }
    }

    fn begin_render_pass(
        &mut self,
        command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>,
        image_num: u32
    ) {
        // begin main render pass
        let clear_values = vec![
            Some([0.1, 0.2, 0.2, 1.0].into()),
            Some([0.0, 0.0, 0.0, 0.0].into()),
            Some([0.0, 0.0, 0.0, 0.0].into()),
            Some(1.0f32.into()),
        ];
        // create render pass begin info struct
        let render_pass_begin_info = RenderPassBeginInfo{
            clear_values: clear_values,
            ..RenderPassBeginInfo::framebuffer(
                self.scene_state.get_framebuffer(image_num)
            )
        };
        log::debug!("Telling cmd buffer builder to enter diffuse render pass");

        // tell builder to begin render pass
        command_buffer_builder
            .begin_render_pass(
                render_pass_begin_info,
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();
        log::debug!("Done beginning render pass");
    }

    // render steps
    fn prep_swapchain(
        &mut self,
    )-> Result<(u32, SwapchainAcquireFuture), AcquireError>
    {
        log::debug!("Prepping swapchain");

        if self.recreate_swapchain {
            log::debug!("Swapchain was suboptimal.");
            self.recreate_swapchain();
        }

        // acquire an image from the swapchain
        let (image_num, suboptimal, acquire_future) = self.acquire_swapchain_image()?;
        log::debug!("Got image from swapchain {:?}", image_num);

        if suboptimal {
            self.recreate_swapchain = true;
        }

        Ok((image_num, acquire_future))
    }

    pub fn get_auto_command_buffer_builder(&self) -> AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>{
        // create a command buffer builder
        let builder = AutoCommandBufferBuilder::primary(
            &self.allocators.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        builder
    }

    // returns the required winit extensions and the required extensions of my choosing
    pub fn get_required_extensions(library: &VulkanLibrary) -> (InstanceExtensions, DeviceExtensions) {
        // what extensions do we need to have in vulkan to draw a window
        let required_extensions = vulkano_win::required_extensions(&library);

        // choose the logical device extensions we're going to use
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        (required_extensions, device_extensions)
    }

    // creates a surface and ties it to the event loop
    pub fn create_event_loop_and_surface(instance: Arc<Instance>) -> (EventLoop<()>, Arc<vulkano::swapchain::Surface>) {
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_title("Ember")
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();
        (event_loop, surface)
    }

    // gets physical GPU and queues
    pub fn get_physical_device_and_queue_family_index(
        instance: &Arc<Instance>,
        device_extensions: DeviceExtensions,
        surface: Arc<vulkano::swapchain::Surface>
    ) -> (Arc<PhysicalDevice>, u32) {
        // get our physical device and queue family
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| { // filter to devices that contain desired features
                p.supported_extensions().contains(&device_extensions)
            })
            .filter_map(|p| { // filter queue families to ones that support graphics
                p.queue_family_properties() // TODO : pick beter queue families since this is one single queue
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        // We select a queue family that supports graphics operations. When drawing to
                        // a window surface, as we do in this example, we also need to check that queues
                        // in this queue family are capable of presenting images to the surface.
                        // q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        //     && p.surface_support(i as u32, &surface).unwrap_or(false)
                        q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    // The code here searches for the first queue family that is suitable. If none is
                    // found, `None` is returned to `filter_map`, which disqualifies this physical
                    // device.
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| {
                // We assign a lower score to device types that are likely to be faster/better.
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }
            })
            .expect("No suitable physical device found");

            (physical_device, queue_family_index)
    }

    // create logical device and queues. Currently a very thin pass-through
    // but it's here in case i ever want to extend this
    pub fn get_logical_device_and_queues(
        physical_device: Arc<PhysicalDevice>,
        device_extensions: DeviceExtensions,
        queue_family_index: u32,
    ) -> (Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>){
        // now create logical device and queues
        let (device, queues) = Device::new(
            physical_device.into(),
            DeviceCreateInfo{
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ).unwrap();
        (device, queues)
    }

    // Create swapchain and images
    pub fn create_swapchain_and_images(
        physical_device: Arc<PhysicalDevice>,
        surface: Arc<vulkano::swapchain::Surface>,
        device: Arc<Device>,
        _queue: Arc<Queue>
    ) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
        let surface_capabilities = physical_device.surface_capabilities(&surface, Default::default()).unwrap();
        
        let image_format = Some(Format::B8G8R8A8_SRGB);
        // let image_format = Some(
        //     device
        //         .physical_device()
        //         .surface_formats(&surface, Default::default())
        //         .unwrap()[0]
        //         .0,
        // );
        log::info!("Swapchaain format {:?}", image_format);

        let binding = surface.clone();
        let window = binding.object().unwrap().downcast_ref::<Window>().unwrap();
        
        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo{
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage {
                    color_attachment: true,
                    ..ImageUsage::empty()
                },
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            }
        ).expect("Couldn't create swapchain.")
    }

    // if the swapchain needs to be recreated
    pub fn recreate_swapchain(&mut self){
        log::debug!("Recreating swapchain...");
        let binding = self.surface;
        let window = binding.object().unwrap().downcast_ref::<Window>().unwrap();
        let _dimensions: [u32; 2] = window.inner_size().into();
        let (new_swapchain, new_images) =
        match self.swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: window.inner_size().into(),
                ..self.swapchain.clone().create_info()
            }) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.recreate_swapchain = false;
        self.scene_state.scale_scene_state_to_images(&new_images);

        // convert images to image views
        let new_images = new_images
            .into_iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>();

        self.swapchain = new_swapchain;
        self.images = new_images;
    } // end of if on swapchain recreation

    // acquires the next swapchain image
    pub fn acquire_swapchain_image(&mut self) -> Result<(u32, bool, SwapchainAcquireFuture), AcquireError> {
        match swapchain::acquire_next_image(self.swapchain, None) {
            Ok(r) => Ok(r),
            Err(AcquireError::OutOfDate) => {
                self.recreate_swapchain();
                self.acquire_swapchain_image()
            }
            Err(e) => Err(e)//panic!("Failed to acquire next image: {:?}", e),
        }
    }

    // create an egui painter
    pub fn initialize_egui(&self) -> (Context, egui_vulkano::Painter){
        let egui_ctx = egui::Context::default();
        let egui_painter = egui_vulkano::Painter::new(
            self.device.clone(),
            self.allocators.memory_allocator.clone(),
            self.allocators.descriptor_set_allocator.clone(),
            self.queue.clone(),
            self.scene_state.ui_pass.clone()
        )
        .unwrap();

        (egui_ctx, egui_painter)
    }

    pub fn create_egui_winit_state<E>(&self, event_loop: &EventLoopWindowTarget<E>) -> egui_winit::State {
        egui_winit::State::new(event_loop)
    }
}

fn submit_render_system_command_buffers_to_render_pass(scene: &mut Scene<Active>, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>) {
    log::debug!("Submitting subsystem secondary commands to render pass");
    let mut world = scene.get_world().unwrap();
    let mut secondary_buffers = world.get_resource_mut::<TriangleSecondaryBuffers>().expect("Couldn't get secondary buffer vec.");
    for buff in secondary_buffers.buffers.drain(..){
        log::debug!("Submitting draw cmd buffer");
        command_buffer_builder.execute_commands(buff).expect("Failed to execute command");
    }
    log::debug!("Moving to lighting pass");
    command_buffer_builder.next_subpass(SubpassContents::SecondaryCommandBuffers).expect("Couldn't step to deferred subpass.");
    let mut lighting_secondary_buffers = world.get_resource_mut::<LightingSecondaryBuffers>().expect("Couldn't get lighting buffer vec");
    for buff in lighting_secondary_buffers.buffers.drain(..){
        log::debug!("Submitting lighting  buffer");
        command_buffer_builder.execute_commands(buff).expect("Failed to execute command");
    }
}