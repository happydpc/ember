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
        QueueCreateInfo
    },
    swapchain::{
        AcquireError,
        Swapchain,
        SwapchainCreationError,
        SwapchainAcquireFuture,
        SwapchainCreateInfo,
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
        SecondaryCommandBuffer,
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
use egui;

use egui::Context;


// std imports
use std::sync::{Arc};

// math
use ember_math::Matrix4f;

// logging
use log;

pub type Aspect = [u32; 2];
pub type SwapchainImageNum = usize;
pub struct TriangleSecondaryBuffers{pub buffers: Vec<Box<dyn SecondaryCommandBuffer>>}
pub struct LightingSecondaryBuffers{pub buffers: Vec<Box<dyn SecondaryCommandBuffer>>}
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
    pub surface: Option<Arc<vulkano::swapchain::Surface<winit::window::Window>>>,
    pub device: Option<Arc<Device>>,
    pub queue: Option<Arc<Queue>>,
    pub swapchain: Option<Arc<Swapchain<winit::window::Window>>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub images: Option<Vec<Arc<ImageView<SwapchainImage<winit::window::Window>>>>>,
    pub scene_state: Option<Arc<SceneState>>,
    pub memory_allocator: Option<Arc<StandardMemoryAllocator>>,
}

impl RenderManager{
    pub fn startup(&mut self) -> (EventLoop<()>, Arc<vulkano::swapchain::Surface<winit::window::Window>>){
        log::info!("Starting RenderManager...");

        // get extensions
        let (required_extensions, device_extensions) = RenderManager::get_required_extensions();

        // create an instance of vulkan with the required extensions
        let instance = Instance::new(
            // None, Version::V1_1, &required_extensions, None
            InstanceCreateInfo{
                enabled_extensions: required_extensions,
                ..Default::default()
            }
        ).unwrap();

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

        // create our memory allocator which I think allocates command buffers (?). docs say it's general use 
        // https://docs.rs/vulkano/latest/vulkano/memory/allocator/type.StandardMemoryAllocator.html
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

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
        scene_state.initialize(swapchain.clone(), device.clone());
        scene_state.scale_scene_state_to_images(images[0].clone(), device.clone());

        let _recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // clone the surface so we can return this clone
        let return_surface = surface.clone();        

        // fill options with initialized values
        self.required_extensions = Some(required_extensions);
        self.device_extensions = Some(device_extensions);
        self.instance = Some(instance);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.swapchain = Some(swapchain);
        self.previous_frame_end = previous_frame_end;
        self.recreate_swapchain = false;
        self.images = Some(images);
        self.scene_state = Some(Arc::new(scene_state));
        self.memory_allocator = Some(memory_allocator);
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
        let egui_winit = self.create_egui_winit_state();
        let egui_state = EguiState{ctx: egui_ctx, painter: egui_painter};
        let secondary_buffer_vec: TriangleSecondaryBuffers = TriangleSecondaryBuffers{buffers: Vec::new()}; 
        let lighting_buffer_vec: LightingSecondaryBuffers = LightingSecondaryBuffers{buffers: Vec::new()};
        let camera_state: [Matrix4f; 2] = [Matrix4f::from_scale(1.0), Matrix4f::from_scale(1.0)];
        let save: bool = false;
        scene.insert_resource(secondary_buffer_vec); // renderable vec to fill
        scene.insert_resource(lighting_buffer_vec);
        scene.insert_resource(save);
        scene.insert_resource(egui_state);
        scene.insert_resource(egui_winit);
        scene.insert_resource(self.device());
        scene.insert_resource(self.surface());
        scene.insert_resource(self.queue());
        scene.insert_resource(camera_state);
        scene.insert_resource(self.scene_state());
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
            device: None,
            queue: None,
            swapchain: None,
            recreate_swapchain: false,
            previous_frame_end: None,
            images: None,
            scene_state: None,
            memory_allocator: None,
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
        puffin::profile_function!();
        log::debug!("Entering draw");
        // create primary command buffer builder
        let mut command_buffer_builder = self.get_auto_command_buffer_builder();

        // get swapchain image num and future
        let (image_num, acquire_future) = self.prep_swapchain();

        // begin main render pass
        log::debug!("Entering main render pass");
        let clear_values = vec![
            [0.1, 0.2, 0.2, 1.0].into(),
            [0.0, 0.0, 0.0, 0.0].into(),
            [0.0, 0.0, 0.0, 0.0].into(),
            1.0f32.into(),
        ];

        // scales framebuffer and attachments to swapchain image view of swapchain image
        self.scene_state().scale_scene_state_to_images(self.images()[image_num].clone(), self.device());

        // insert stuff into scene that systems will need
        let secondary_buffer_vec: TriangleSecondaryBuffers = TriangleSecondaryBuffers{buffers: Vec::new()}; 
        let lighting_buffer_vec: LightingSecondaryBuffers = LightingSecondaryBuffers{buffers: Vec::new()};
        scene.insert_resource(secondary_buffer_vec); // renderable vec to fill
        scene.insert_resource(lighting_buffer_vec);
        let save: bool = false;
        scene.insert_resource(save);
        scene.insert_resource(image_num); // insert image
        self.insert_render_data_into_scene(scene); // inserts vulkan resources into scene

        // start egui frame
        {
            let mut world = scene.get_world().unwrap();
            let ctx = world.get_resource_mut::<EguiState>().expect("Couldn't get egui state.").ctx.clone();
            let mut egui_winit = world.get_resource_mut::<egui_winit::State>().expect("Couldn't get egui winit state.");
            ctx.begin_frame(egui_winit.take_egui_input(self.surface().window()));
        }

        // run all systems. This will build secondary command buffers
        log::debug!("----Render schedule-----");
        scene.run_render_schedule();

        let save = {
            *scene.get_world().unwrap().get_resource::<bool>().expect("Couldn't get save bool")
        };
        if save {
            scene.serialize();
        }

        // get egui shapes from world
        log::debug!("Getting egui shapes");
        let egui_output = {
            // let world = scene.get_world().unwrap();
            let ctx = scene.get_world().unwrap().get_resource_mut::<EguiState>().expect("Couldn't get egui state").ctx.clone();
            let egui_output = ctx.end_frame();
            let platform_output = egui_output.platform_output.clone();
            let textures_delta = egui_output.textures_delta.clone();

            // let mut egui_winit = scene.get_world().unwrap().get_resource_mut::<egui_winit::State>().expect("Couldn't get egui winit state");
            scene
                .get_world()
                .unwrap()
                .get_resource_mut::<egui_winit::State>()
                .expect("Couldn't get egui winit state")
                .handle_platform_output(
                    self.surface().window(),
                    &ctx,
                    platform_output
                );
            
            scene
                .get_world()
                .unwrap()
                .get_resource_mut::<EguiState>()
                .expect(".")
                .painter
                .update_textures(textures_delta, &mut command_buffer_builder)
                .expect("egui texture error");
            egui_output
        };

        // tell builder to begin render pass
        command_buffer_builder
            .begin_render_pass(
                self.scene_state().get_framebuffer_image(image_num),
                SubpassContents::SecondaryCommandBuffers,
                clear_values,
            )
            .unwrap();

        // get and submit secondary command buffers to render pass
        {
            let mut world = scene.get_world().unwrap();
            let mut secondary_buffers = world.get_resource_mut::<TriangleSecondaryBuffers>().expect("Couldn't get secondary buffer vec.");
            // submit secondary buffers
            for buff in secondary_buffers.buffers.drain(..){
                command_buffer_builder.execute_commands(buff).expect("Failed to execute command");
            }

            command_buffer_builder.next_subpass(SubpassContents::SecondaryCommandBuffers).expect("Couldn't step to deferred subpass.");

            let mut lighting_secondary_buffers = world.get_resource_mut::<LightingSecondaryBuffers>().expect("Couldn't get lighting buffer vec");
            for buff in lighting_secondary_buffers.buffers.drain(..){
                command_buffer_builder.execute_commands(buff).expect("Failed to execute command");
            }
        }

        // add egui draws to command buffer
        {
            let surface = self.surface();
            let size = surface.window().inner_size();
            let sf: f32 = surface.window().scale_factor() as f32;
            // let sf = 1.0;
            let mut world = scene.get_world().unwrap();
            let ctx = world.get_resource_mut::<EguiState>().expect("Couldn't get egui state.").ctx.clone();
            // ctx.set_pixels_per_point(1.0);
            command_buffer_builder.set_viewport(0, [self.scene_state().viewport()]);
            world
                .get_resource_mut::<EguiState>()
                .expect("Couldn't get egui state.")
                .painter
                .draw(
                    &mut command_buffer_builder,
                    [(size.width as f32) / sf, (size.height as f32) / sf],
                    &ctx,
                    egui_output.shapes,
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
    fn prep_swapchain(
        &mut self,
    )->(usize, SwapchainAcquireFuture<winit::window::Window>)
    {
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

    // insert required render data into scene so systems can run
    pub fn insert_render_data_into_scene(&mut self, scene: &mut Scene<Active>) {
        let camera_state: [Matrix4f; 2] = [Matrix4f::from_scale(1.0), Matrix4f::from_scale(1.0)];
        // insert resources. some of these should eventually be submitted more often than othrs
        scene.insert_resource(self.device());
        scene.insert_resource(self.surface());
        scene.insert_resource(self.queue());
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
            .build(&event_loop, instance.clone())
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
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .filter(|&p| { // filter to devices that contain desired features
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
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
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
        physical_device: PhysicalDevice,
        device_extensions: &DeviceExtensions,
        queue_family: QueueFamily,
    ) -> (Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>){
        // now create logical device and queues
        let (device, queues) = Device::new(
            physical_device,
            DeviceCreateInfo{
                enabled_extensions: physical_device.required_extensions().union(&device_extensions),
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)], 
                ..Default::default()
            },
        ).unwrap();
        (device, queues)
    }

    // Create swapchain and images
    pub fn create_swapchain_and_images(
        physical_device: PhysicalDevice,
        surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>,
        device: Arc<Device>,
        _queue: Arc<Queue>
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let surface_capabilities = physical_device.surface_capabilities(&surface, Default::default()).unwrap();
        
        let image_format = Some(
            physical_device
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );
        let _dimensions: [u32; 2] = surface.window().inner_size().into();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo{
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: surface.window().inner_size().into(),
                image_usage: ImageUsage::color_attachment(),
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
        let _dimensions: [u32; 2] = self.surface().window().inner_size().into();
        let (new_swapchain, new_images) =
        match self.swapchain()
            .recreate(SwapchainCreateInfo {
                image_extent: self.surface().window().inner_size().into(),
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
    pub fn initialize_egui(&self) -> (Context, egui_vulkano::Painter){
        let egui_ctx = egui::Context::default();
        let egui_painter = egui_vulkano::Painter::new(
            self.device(),
            self.memory_allocator(),
            self.queue(),
            Subpass::from(self.scene_state().render_passes[0].clone(), 2).unwrap(),
        )
        .unwrap();

        (egui_ctx, egui_painter)
    }

    pub fn create_egui_winit_state(&self) -> egui_winit::State{
        let surface = self.surface();
        let window = surface.window();
        egui_winit::State::new(4096, window)
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

    pub fn surface(&self) -> Arc<vulkano::swapchain::Surface<winit::window::Window>> {
        self.surface.clone().unwrap().clone()
    }

    pub fn swapchain(&self) -> Arc<Swapchain<winit::window::Window>> {
        self.swapchain.clone().unwrap().clone()
    }

    pub fn images(&self) -> Vec<Arc<ImageView<SwapchainImage<winit::window::Window>>>> {
        self.images.clone().unwrap().clone()
    }

    pub fn scene_state(&self) -> Arc<SceneState> {
        self.scene_state.clone().unwrap().clone()
    }
}