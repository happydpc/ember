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
    }
};


use puffin;

// Vulkano imports
use vulkano::{
    VulkanLibrary,
    instance::{
        Instance,
        InstanceExtensions,
        InstanceCreateInfo,
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
        QueueFlags
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
    render_pass::{
        Subpass,
    },
    pipeline::{
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
        SecondaryAutoCommandBuffer,
        RenderPassBeginInfo,
        allocator::StandardCommandBufferAllocator,
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

pub type Aspect = [u32; 2];
pub type SwapchainImageNum = usize;
pub struct TriangleSecondaryBuffers{pub buffers: Vec<Box<SecondaryAutoCommandBuffer>>}
pub struct LightingSecondaryBuffers{pub buffers: Vec<Box<SecondaryAutoCommandBuffer>>}
pub struct DiffuseBuffer{pub buffer: Arc<ImageView<AttachmentImage>>}
pub struct DepthBuffer{pub buffer: Arc<ImageView<AttachmentImage>>}
pub struct NormalsBuffer{pub buffer: Arc<ImageView<AttachmentImage>>}

pub type DirectionalLightingPipelne = GraphicsPipeline;
pub type AmbientLightingPipeline = GraphicsPipeline;
pub type PointLightingPipeline = GraphicsPipeline;

pub struct RenderManager{
    // Vulkan
    required_extensions: Option<InstanceExtensions>,
    device_extensions: Option<DeviceExtensions>,
    instance: Option<Arc<Instance>>,
    pub surface: Option<Arc<vulkano::swapchain::Surface>>,
    pub window: Option<Arc<winit::window::Window>>,
    pub device: Option<Arc<Device>>,
    pub queue: Option<Arc<Queue>>,
    pub swapchain: Option<Arc<Swapchain>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub images: Option<Vec<Arc<ImageView<SwapchainImage>>>>,
    pub scene_state: Option<Arc<SceneState>>,
    pub memory_allocator: Option<Arc<StandardMemoryAllocator>>,
    pub command_buffer_allocator: Option<Arc<StandardCommandBufferAllocator>>,
    pub descriptor_set_allocator: Option<Arc<StandardDescriptorSetAllocator>>,
}

impl RenderManager{
    pub fn startup(&mut self) -> (EventLoop<()>, Arc<vulkano::swapchain::Surface>){
        log::info!("Starting RenderManager...");

        // get library
        let library = VulkanLibrary::new().unwrap();

        // get extensions
        let (required_extensions, device_extensions) = RenderManager::get_required_extensions(&library);

        // create an instance of vulkan with the required extensions
        let instance = Instance::new(
            library,
            // None, Version::V1_1, &required_extensions, None
            InstanceCreateInfo{
                enabled_extensions: required_extensions,
                enumerate_portability: true,
                ..Default::default()
            }
        ).unwrap();

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
            physical_device,
            device_extensions.clone(),
            queue_family_index
        );

        // create our memory allocator which I think allocates command buffers (?). docs say it's general use 
        // https://docs.rs/vulkano/latest/vulkano/memory/allocator/type.StandardMemoryAllocator.html
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let cmd_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone()));

        // get queue
        let queue = queues.next().unwrap();

        // create swapchain, images
        let (swapchain, images) = RenderManager::create_swapchain_and_images(
            physical_device,
            surface.clone(),
            device.clone(),
            queue.clone()
        );

        // store swapchain images?
        let images = images
            .into_iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>();

        // TODO : Somehow make this aware of when scenes are Active and do this there instead.
        let mut scene_state = SceneState::new();
        scene_state.initialize(swapchain.clone(), device.clone(), memory_allocator.clone());
        scene_state.scale_scene_state_to_images(images[0].clone(), device.clone());

        let _recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // clone the surface so we can return this clone
        let return_surface = surface.clone();        

        let window = surface.clone().object().unwrap().downcast_ref::<Window>().unwrap();

        // fill options with initialized values
        self.required_extensions = Some(required_extensions);
        self.device_extensions = Some(device_extensions);
        self.instance = Some(instance);
        self.surface = Some(surface);
        self.window = Some(Arc::new(*window));
        self.device = Some(device);
        self.queue = Some(queue);
        self.swapchain = Some(swapchain);
        self.previous_frame_end = previous_frame_end;
        self.recreate_swapchain = false;
        self.images = Some(images);
        self.scene_state = Some(Arc::new(scene_state));
        self.memory_allocator = Some(memory_allocator);
        self.command_buffer_allocator = Some(cmd_buffer_allocator);
        self.descriptor_set_allocator = Some(descriptor_set_allocator);
        (event_loop, return_surface)
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
        // get required egui data
        let (egui_ctx, egui_painter) = self.initialize_egui();
        // let egui_winit = self.create_egui_winit_state();
        let egui_state = EguiState{ctx: egui_ctx, painter: egui_painter};
        let secondary_buffer_vec: TriangleSecondaryBuffers = TriangleSecondaryBuffers{buffers: Vec::new()}; 
        let lighting_buffer_vec: LightingSecondaryBuffers = LightingSecondaryBuffers{buffers: Vec::new()};
        let camera_state: [Matrix4f; 2] = [Matrix4f::from_scale(1.0), Matrix4f::from_scale(1.0)];
        let save: bool = false;
        scene.insert_resource(secondary_buffer_vec); // renderable vec to fill
        scene.insert_resource(lighting_buffer_vec);
        scene.insert_resource(save);
        scene.insert_resource(egui_state);
        // scene.insert_resource(egui_winit);
        scene.insert_resource(self.device());
        scene.insert_resource(self.surface());
        scene.insert_resource(self.queue());
        scene.insert_resource(camera_state);
        scene.insert_resource(self.scene_state());
        scene.insert_resource(self.device());
        scene.insert_resource(self.surface());
        scene.insert_resource(self.queue());
        scene.insert_resource(self.memory_allocator());
        scene.insert_resource(self.descriptor_set_allocator());
        scene.insert_resource(self.command_buffer_allocator());
        log::debug!("Does device exist: {:?}", scene.contains_resource::<Arc<Device>>());
    }

    // create a new render manager with Inactive values
    pub fn new() -> Self {
        log::info!("Creating RenderManager...");

        // initialize our render system with all of the required vulkan components
        let render_sys = RenderManager{
            // Vulkan
            required_extensions: None,
            device_extensions: None,
            instance: None,
            surface: None,
            window: None,
            device: None,
            queue: None,
            swapchain: None,
            recreate_swapchain: false,
            previous_frame_end: None,
            images: None,
            scene_state: None,
            memory_allocator: None,
            command_buffer_allocator: None,
            descriptor_set_allocator: None,
        };
        render_sys
    }

    // run the render manager
    pub fn run(&mut self) {
        // self.window.run();
    }

    pub fn draw(
        &mut self,
        scene: &mut Scene<Active>
    ){
        log::debug!("Entering draw");

        // create primary command buffer builder
        let mut command_buffer_builder = self.get_auto_command_buffer_builder();

        // get swapchain image num and future
        let (image_num, acquire_future) = self.prep_swapchain();

        // scales framebuffer and attachments to swapchain image view of swapchain image
        self.scene_state().scale_scene_state_to_images(self.images()[image_num as usize].clone(), self.device());

        // insert stuff into scene that systems will need
        self.insert_render_data_into_scene(scene); // inserts vulkan resources into scene

        // start egui frame
        self.start_egui_frame(scene);

        // run all systems. This will build secondary command buffers
        log::debug!("----Render schedule-----");
        scene.run_render_schedule();

        // get egui shapes from world
        log::debug!("Getting egui shapes");
        let egui_output: FullOutput = self.get_egui_output(scene, &mut command_buffer_builder);

        // set clear values and begin the render pass
        self.begin_render_pass(image_num, &mut command_buffer_builder);

        // get and submit secondary command buffers to render pass
        submit_render_system_command_buffers_to_render_pass(scene, &mut command_buffer_builder);

        // add egui draws to command buffer
        self.draw_egui(scene, &mut command_buffer_builder, egui_output);

        // end egui pass
        log::debug!("ending render pass");
        command_buffer_builder.end_render_pass().unwrap();

        // build command buffer
        log::debug!("Building command buffer");
        let command_buffer = command_buffer_builder.build().unwrap();

        // submit and render
        log::debug!("Submitting");
        self.submit_command_buffer_and_render(acquire_future, command_buffer, image_num);
    }

    fn submit_command_buffer_and_render(&mut self, acquire_future: SwapchainAcquireFuture, command_buffer: PrimaryAutoCommandBuffer, image_num: u32) {
        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain(),
                    image_num as u32
                ),
            )
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

    fn draw_egui(&mut self, scene: &mut Scene<Active>, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>, egui_output: FullOutput) {
        let size = self.window().inner_size();
        let sf: f32 = self.window().scale_factor() as f32;
        let mut world = scene.get_world().unwrap();
        let ctx = world.get_resource_mut::<EguiState>().expect("Couldn't get egui state.").ctx.clone();
        command_buffer_builder.set_viewport(0, [self.scene_state().viewport()]);
        world
            .get_resource_mut::<EguiState>()
            .expect("Couldn't get egui state.")
            .painter
            .draw(
                command_buffer_builder,
                [(size.width as f32) / sf, (size.height as f32) / sf],
                &ctx,
                egui_output.shapes,
            )
            .unwrap();
    }

    fn start_egui_frame(&mut self, scene: &mut Scene<Active>) {
        let mut world = scene.get_world().unwrap();
        let ctx = world.get_resource_mut::<EguiState>().expect("Couldn't get egui state.").ctx.clone();
        let mut egui_winit = world.get_resource_mut::<egui_winit::State>().expect("Couldn't get egui winit state.");
        ctx.begin_frame(egui_winit.take_egui_input(&self.window()));
    }

    fn get_egui_output(&mut self, scene: &mut Scene<Active>, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>) -> egui::FullOutput {
        let egui_output = {
            // let world = scene.get_world().unwrap();
            let ctx = scene.get_world().unwrap().get_resource_mut::<EguiState>().expect("Couldn't get egui state").ctx.clone();
            let egui_output = ctx.end_frame();

            let platform_output = egui_output.platform_output.clone();
            scene
                .get_world()
                .unwrap()
                .get_resource_mut::<egui_winit::State>()
                .expect("Couldn't get egui winit state")
                .handle_platform_output(
                    &self.window(),
                    &ctx,
                    platform_output
                );
    
            let textures_delta = egui_output.textures_delta.clone();
            scene
                .get_world()
                .unwrap()
                .get_resource_mut::<EguiState>()
                .expect(".")
                .painter
                .update_textures(textures_delta, command_buffer_builder)
                .expect("egui texture error");
            egui_output
        };
        egui_output
    }

    fn begin_render_pass(&mut self, image_num: u32, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>) {
        // begin main render pass
        log::debug!("Entering main render pass");
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
                self.scene_state().get_framebuffer_image(image_num)
            )
        };
        // tell builder to begin render pass
        command_buffer_builder
            .begin_render_pass(
                render_pass_begin_info,
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();
    }

    // render steps
    fn prep_swapchain(
        &mut self,
    )->(u32, SwapchainAcquireFuture)
    {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // acquire an image from the swapchain
        let (image_num, suboptimal, acquire_future) = self.acquire_swapchain_image();

        if suboptimal {
            self.recreate_swapchain()
        }
        (image_num, acquire_future)
    }

    pub fn get_auto_command_buffer_builder(&self) -> AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>{
        // create a command buffer builder
        let builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator(),
            self.queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        builder
    }

    // insert required render data into scene so systems can run
    pub fn insert_render_data_into_scene(&mut self, scene: &mut Scene<Active>) {
        let camera_state: [Matrix4f; 2] = [Matrix4f::from_scale(1.0), Matrix4f::from_scale(1.0)];
        // insert resources. some of these should eventually be submitted more often than othrs
        scene.insert_resource(camera_state);
        scene.insert_resource(self.scene_state());

        let secondary_buffer_vec: TriangleSecondaryBuffers = TriangleSecondaryBuffers{buffers: Vec::new()}; 
        let lighting_buffer_vec: LightingSecondaryBuffers = LightingSecondaryBuffers{buffers: Vec::new()};
        scene.insert_resource(secondary_buffer_vec); // renderable vec to fill
        scene.insert_resource(lighting_buffer_vec);
    }

    // returns the required winit extensions and the required extensions of my choosing
    pub fn get_required_extensions(library: &VulkanLibrary) -> (InstanceExtensions, DeviceExtensions) {
        // what extensions do we need to have in vulkan to draw a window
        let required_extensions = vulkano_win::required_extensions(&library);

        // choose the logical device extensions we're going to use
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
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
        
        let image_format = Some(
            physical_device
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );
        let window = surface.clone().object().unwrap().downcast_ref::<Window>().unwrap();
        
        let _dimensions: [u32; 2] = window.inner_size().into();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo{
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: _dimensions,
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
        let _dimensions: [u32; 2] = self.window().inner_size().into();
        let (new_swapchain, new_images) =
        match self.swapchain()
            .recreate(SwapchainCreateInfo {
                image_extent: self.window().inner_size().into(),
                ..self.swapchain().create_info()
            }) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.recreate_swapchain = false;

        // convert images to image views
        let new_images = new_images
            .into_iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect::<Vec<_>>();

        self.scene_state().scale_scene_state_to_images(new_images[0].clone(), self.device());

        self.swapchain = Some(new_swapchain);
        self.images = Some(new_images);
    } // end of if on swapchain recreation

    // acquires the next swapchain image
    pub fn acquire_swapchain_image(&mut self) -> (u32, bool, SwapchainAcquireFuture) {
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
    pub fn initialize_egui(&self) -> (Context, egui_vulkano::Painter){
        let egui_ctx = egui::Context::default();
        let egui_painter = egui_vulkano::Painter::new(
            self.device(),
            self.memory_allocator(),
            self.descriptor_set_allocator(),
            self.queue(),
            Subpass::from(self.scene_state().render_passes[0].clone(), 2).unwrap(),
        )
        .unwrap();

        (egui_ctx, egui_painter)
    }

    pub fn create_egui_winit_state<E>(&self, event_loop: &EventLoopWindowTarget<E>) -> egui_winit::State {
        egui_winit::State::new(event_loop)
        // egui_winit::State::from_pixels_per_point(4096, 1.0)
    }

    // getters
    pub fn device(&self) -> Arc<Device> {
        self.device.clone().unwrap().clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone().unwrap().clone()
    }

    pub fn memory_allocator(&self) -> Arc<StandardMemoryAllocator> {
        self.memory_allocator.clone().unwrap().clone()
    }

    pub fn command_buffer_allocator(&self) -> Arc<StandardCommandBufferAllocator> {
        self.command_buffer_allocator.clone().unwrap().clone()
    }

    pub fn descriptor_set_allocator(&self) -> Arc<StandardDescriptorSetAllocator> {
        self.descriptor_set_allocator.clone().unwrap().clone()
    }

    pub fn surface(&self) -> Arc<vulkano::swapchain::Surface> {
        self.surface.clone().unwrap().clone()
    }

    pub fn window(&self) -> Arc<winit::window::Window> {
        self.window.clone().unwrap().clone()
    }

    pub fn swapchain(&self) -> Arc<Swapchain> {
        self.swapchain.clone().unwrap().clone()
    }

    pub fn images(&self) -> Vec<Arc<ImageView<SwapchainImage>>> {
        self.images.clone().unwrap().clone()
    }

    pub fn scene_state(&self) -> Arc<SceneState> {
        self.scene_state.clone().unwrap().clone()
    }
}

fn submit_render_system_command_buffers_to_render_pass(scene: &mut Scene<Active>, command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer, Arc<StandardCommandBufferAllocator>>) {
    let mut world = scene.get_world().unwrap();
    let mut secondary_buffers = world.get_resource_mut::<TriangleSecondaryBuffers>().expect("Couldn't get secondary buffer vec.");
    for buff in secondary_buffers.buffers.drain(..){
        command_buffer_builder.execute_commands(buff).expect("Failed to execute command");
    }
    command_buffer_builder.next_subpass(SubpassContents::SecondaryCommandBuffers).expect("Couldn't step to deferred subpass.");
    let mut lighting_secondary_buffers = world.get_resource_mut::<LightingSecondaryBuffers>().expect("Couldn't get lighting buffer vec");
    for buff in lighting_secondary_buffers.buffers.drain(..){
        command_buffer_builder.execute_commands(buff).expect("Failed to execute command");
    }
}