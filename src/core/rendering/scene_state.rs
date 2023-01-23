use crate::core::scene::Scene;
use crate::core::systems::render_systems::DirectionalLightingSystemPipeline;
use crate::core::systems::render_systems::AmbientLightingSystemPipeline;
use crate::core::systems::render_systems::RenderableDrawSystemPipeline;
use crate::core::systems::terrain_systems::TerrainDrawSystemPipeline;
use crate::core::systems::RequiresGraphicsPipeline;

use vulkano::memory;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::RenderPass;
use vulkano::format::Format;
use vulkano::swapchain::Swapchain;
use vulkano::device::Device;
use vulkano::image::view::ImageView;
use vulkano::image::AttachmentImage;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::FramebufferCreateInfo;
use vulkano::image::SwapchainImage;
use vulkano::image::ImageUsage;
use vulkano::image::ImageAccess;



use std::sync::{Arc, Mutex};
use std::any::TypeId;
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct SceneState{
    pub pipelines: Mutex<HashMap<TypeId, Arc<GraphicsPipeline>>>,
    pub render_passes: Vec<Arc<RenderPass>>,
    pub diffuse_buffer: Arc<Mutex<Arc<ImageView<AttachmentImage>>>>,
    pub normals_buffer: Arc<Mutex<Arc<ImageView<AttachmentImage>>>>,
    pub depth_buffer: Arc<Mutex<Arc<ImageView<AttachmentImage>>>>,
    pub viewport: Arc<Mutex<Viewport>>,
    pub framebuffers: Arc<Mutex<Option<Arc<Framebuffer>>>>,
    memory_allocator: Arc<StandardMemoryAllocator>,
}

impl SceneState{
    pub fn new(swapchain: Arc<Swapchain>, device: Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>){  

        // create pipelines
        let directional_lighting_pipeline = DirectionalLightingSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone());
        let ambient_lighting_pipeline = AmbientLightingSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone());
        let renderable_pipeline = RenderableDrawSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone());
        let terrain_draw_pipeline = TerrainDrawSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone());
        
        // create viewport
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };


        // add passes
        let mut render_passes = Vec::new();
        let pass = SceneState::build_render_pass(swapchain.image_format(), device.clone());
        render_passes.push(pass);
        
        // add pipelines
        let mut pipelines = HashMap::new();
        pipelines.insert(TypeId::of::<DirectionalLightingSystemPipeline>(), directional_lighting_pipeline);
        pipelines.insert(TypeId::of::<RenderableDrawSystemPipeline>(), renderable_pipeline);
        pipelines.insert(TypeId::of::<AmbientLightingSystemPipeline>(), ambient_lighting_pipeline);
        pipelines.insert(TypeId::of::<TerrainDrawSystemPipeline>(), terrain_draw_pipeline);
        let pipeline_mutex = Mutex::new(pipelines);

        // create buffers
        let (
            diffuse_buffer,
            normals_buffer,
            depth_buffer
        ) = SceneState::build_buffers(device.clone(), memory_allocator);

        let mut scene_state_instance: SceneState = SceneState{
            pipelines: pipeline_mutex,
            render_passes: render_passes,
            diffuse_buffer: Arc::new(Mutex::new((diffuse_buffer))),
            normals_buffer: Arc::new(Mutex::new((normals_buffer))),
            depth_buffer: Arc::new(Mutex::new((depth_buffer))),
            viewport: Arc::new(Mutex::new(viewport)),
            framebuffers: Arc::new(Mutex::new(None)),
            memory_allocator: memory_allocator.clone()
        };

    }

    fn build_render_pass(swapchain_format: Format, device: Arc<Device>) -> Arc<RenderPass> {
        let render_pass = vulkano::ordered_passes_renderpass!(device.clone(),
                attachments: {
                    // The image that will contain the final rendering (in this example the swapchain
                    // image, but it could be another image).
                    final_color: {
                        load: Clear,
                        store: Store,
                        format: swapchain_format,
                        samples: 1,
                    },
                    // Will be bound to `self.diffuse_buffer`.
                    diffuse: {
                        load: Clear,
                        store: DontCare,
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
                    },
                    // ui
                    { 
                        color: [final_color],
                        depth_stencil: {depth},
                        input: []
                    }
                ]
            )
            .unwrap();
        render_pass
    }

    fn build_buffers(device: Arc<Device>, memory_allocator: Arc<StandardMemoryAllocator>)
    -> (Arc<ImageView<AttachmentImage>>, Arc<ImageView<AttachmentImage>>, Arc<ImageView<AttachmentImage>>){
        // TODO: use shortcut provided in vulkano 0.6
        let diffuse_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                &memory_allocator,
                [1, 1],
                Format::A2B10G10R10_UNORM_PACK32,
                ImageUsage {
                    transient_attachment: true,
                    input_attachment: true,
                    ..ImageUsage::empty()
                },
            )
            .unwrap(),
        )
        .unwrap();
        let normals_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                &memory_allocator,
                [1, 1],
                Format::R16G16B16A16_SFLOAT,
                ImageUsage {
                    transient_attachment: true,
                    input_attachment: true,
                    ..ImageUsage::empty()
                },
            )
            .unwrap(),
        )
        .unwrap();
        let depth_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                &memory_allocator,
                [1, 1],
                Format::D16_UNORM,
                ImageUsage {
                    transient_attachment: true,
                    input_attachment: true,
                    ..ImageUsage::empty()
                },
            )
            .unwrap(),
        )
        .unwrap();
        (diffuse_buffer, normals_buffer, depth_buffer)
    }

    pub fn scale_scene_state_to_images(&self, image: Arc<ImageView<SwapchainImage>>, device: Arc<Device>){
        self.scale_framebuffers_to_images(image.clone(), device.clone());
        self.rescale_viewport(image.clone());
    }

    fn rescale_viewport(&self, image: Arc<ImageView<SwapchainImage>>){
        let dimensions = image.image().dimensions().width_height();
        self.viewport.try_lock().unwrap().dimensions = [dimensions[0] as f32, dimensions[1] as f32]
    }

    fn scale_framebuffers_to_images(&self, image: Arc<ImageView<SwapchainImage>>, device: Arc<Device>){
        // should probably comment some of this i guess
        let dimensions = image.clone().image().dimensions().width_height();
        if self.diffuse_buffer().image().dimensions().width_height() != dimensions {
            let diffuse_buffer = ImageView::new_default(
                AttachmentImage::with_usage(
                    &self.memory_allocator,
                    dimensions,
                    Format::A2B10G10R10_UNORM_PACK32,
                    ImageUsage {
                        transient_attachment: true,
                        input_attachment: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap(),
            )
            .unwrap();
            let normals_buffer = ImageView::new_default(
                AttachmentImage::with_usage(
                    &self.memory_allocator,
                    dimensions,
                    Format::R16G16B16A16_SFLOAT,
                    ImageUsage {
                        transient_attachment: true,
                        input_attachment: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap(),
            )
            .unwrap();
            let depth_buffer = ImageView::new_default(
                AttachmentImage::with_usage(
                    &self.memory_allocator,
                    dimensions,
                    Format::D16_UNORM,
                    ImageUsage {
                        transient_attachment: true,
                        input_attachment: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap(),
            )
            .unwrap();

            let framebuffer = Framebuffer::new(
                self.render_passes[0].clone(),
                FramebufferCreateInfo {
                    attachments: vec![
                        image.clone(),
                        diffuse_buffer.clone(),
                        normals_buffer.clone(),
                        depth_buffer.clone(),
                    ],
                    ..Default::default()
                },
            ).expect("Couldn't create framebuffer");
            
            *self.framebuffers.lock().unwrap() = Some(framebuffer);
            *self.diffuse_buffer.lock().unwrap() = diffuse_buffer;
            *self.normals_buffer.lock().unwrap() = normals_buffer;
            *self.depth_buffer.lock().unwrap() = depth_buffer;
        }
    }

    pub fn get_pipeline_for_system<S: 'static>(&self) -> Option<Arc<GraphicsPipeline>>{
        match self.pipelines.lock().unwrap().get(&TypeId::of::<S>()) {
            Some(pipeline) => Some(pipeline.clone()),
            None => None
        }
    }

    pub fn set_pipeline_for_system<S: 'static>(&self, pipeline: Arc<GraphicsPipeline>){
        self.pipelines.lock().unwrap().insert(TypeId::of::<S>(), pipeline);
    }

    pub fn get_framebuffer_image(&self, _image_num: u32) -> Arc<Framebuffer> {
        // todo : is there only one image???
        self.framebuffers.clone().lock().unwrap().clone().unwrap()
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport.lock().unwrap().clone()
    }

    pub fn diffuse_buffer(&self) -> Arc<ImageView<AttachmentImage>> {
        self.diffuse_buffer.lock().unwrap().clone()
    }

    pub fn normals_buffer(&self) -> Arc<ImageView<AttachmentImage>> {
        self.normals_buffer.lock().unwrap().clone()
    }

    pub fn depth_buffer(&self) -> Arc<ImageView<AttachmentImage>> {
        self.depth_buffer.lock().unwrap().clone()
    }
   
}