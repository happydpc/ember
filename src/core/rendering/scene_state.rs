use crate::core::systems::render_systems::DirectionalLightingSystemPipeline;
use crate::core::systems::render_systems::AmbientLightingSystemPipeline;
use crate::core::systems::render_systems::RenderableDrawSystemPipeline;
use crate::core::systems::terrain_systems::TerrainDrawSystemPipeline;
use crate::core::systems::RequiresGraphicsPipeline;

use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::RenderPass;
use vulkano::format::Format;
use vulkano::render_pass::Subpass;
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
use std::thread::current;

use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct SceneState{
    pub pipelines: Mutex<HashMap<TypeId, Arc<GraphicsPipeline>>>,
    pub render_passes: Vec<Arc<RenderPass>>,
    pub diffuse_buffer: Arc<Mutex<Arc<ImageView<AttachmentImage>>>>,
    pub normals_buffer: Arc<Mutex<Arc<ImageView<AttachmentImage>>>>,
    pub depth_buffer: Arc<Mutex<Arc<ImageView<AttachmentImage>>>>,
    pub viewport: Arc<Mutex<Viewport>>,
    pub framebuffers: Arc<Mutex<Vec<Arc<Framebuffer>>>>,
    pub diffuse_pass: Subpass,
    pub lighting_pass: Subpass,
    pub ui_pass: Subpass,
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
}

impl SceneState{
    pub fn new(
        swapchain: Arc<Swapchain>,
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>
    ) -> Arc<Self> {  
        
        // create viewport
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [1.0, 1.0],
            depth_range: 0.0..1.0,
        };


        // add passes
        let mut render_passes = Vec::new();
        let pass = SceneState::build_render_pass(swapchain.image_format(), device.clone());

        let diffuse_pass = Subpass::from(pass.clone(), 0).unwrap();
        let lighting_pass = Subpass::from(pass.clone(), 1).unwrap();
        let ui_pass = Subpass::from(pass.clone(), 2).unwrap();

        // create pipelines
        let directional_lighting_pipeline = DirectionalLightingSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());
        let ambient_lighting_pipeline = AmbientLightingSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());
        let renderable_pipeline = RenderableDrawSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());
        let terrain_draw_pipeline = TerrainDrawSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());

        // now that we've used the render pass to build pipelines, push it
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
        ) = SceneState::build_buffers(memory_allocator.clone());

        let scene_state_instance: SceneState = SceneState{
            pipelines: pipeline_mutex,
            render_passes: render_passes,
            diffuse_buffer: Arc::new(Mutex::new(diffuse_buffer)),
            normals_buffer: Arc::new(Mutex::new(normals_buffer)),
            depth_buffer: Arc::new(Mutex::new(depth_buffer)),
            viewport: Arc::new(Mutex::new(viewport)),
            framebuffers: Arc::new(Mutex::new(Vec::new())),
            diffuse_pass: diffuse_pass,
            lighting_pass: lighting_pass,
            ui_pass: ui_pass,
            device: device.clone(),
            memory_allocator: memory_allocator.clone()
        };

        Arc::new(scene_state_instance)

    }

    fn build_render_pass(swapchain_format: Format, device: Arc<Device>) -> Arc<RenderPass> {
        let render_pass = vulkano::ordered_passes_renderpass!(
            device.clone(),
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
                    depth_stencil: {},
                    input: []
                }
            ]
        )
        .unwrap();
        render_pass
    }

    fn build_buffers(memory_allocator: Arc<StandardMemoryAllocator>)
    -> (Arc<ImageView<AttachmentImage>>, Arc<ImageView<AttachmentImage>>, Arc<ImageView<AttachmentImage>>){
        // TODO: use shortcut provided in vulkano 0.6
        let diffuse_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                &memory_allocator,
                [1, 1],
                // Format::A2B10G10R10_UNORM_PACK32,
                Format::B8G8R8A8_SRGB,
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

    pub fn scale_scene_state_to_images(&self, images: &[Arc<SwapchainImage>]){
        self.scale_image_views_to_images(images);
        self.recreate_framebuffers(images);
        self.rescale_viewport(images);
    }

    fn recreate_framebuffers(&self, images: &[Arc<SwapchainImage>]){
        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                let fb = Framebuffer::new(
                    self.render_passes[0].clone(),
                    FramebufferCreateInfo {
                        attachments: vec![
                            ImageView::new_default(image.clone()).unwrap(),
                            self.diffuse_buffer(),
                            self.normals_buffer(),
                            self.depth_buffer(),
                        ],
                        ..Default::default()
                    },
                )
                .unwrap();
                fb
            })
            .collect::<Vec<_>>();

        // let framebuffer = Framebuffer::new(
        //     self.render_passes[0].clone(),
        //     FramebufferCreateInfo {
        //         attachments: vec![
        //             ImageView::new_default(image.clone()).unwrap(),
        //             self.diffuse_buffer(),
        //             self.normals_buffer(),
        //             self.depth_buffer(),
        //         ],
        //         ..Default::default()
        //     },
        // ).expect("Couldn't create framebuffer");
        *self.framebuffers.lock().unwrap()  = framebuffers;

    }

    fn rescale_viewport(&self, images: &[Arc<SwapchainImage>]){
        let image = images[0].clone();
        let dimensions = image.dimensions().width_height();
        {
            self.viewport.lock().unwrap().dimensions = [dimensions[0] as f32, dimensions[1] as f32];
        }
        self.match_pipelines_to_viewport_state(self.viewport());
    }

    fn match_pipelines_to_viewport_state(&self, viewport: Viewport){

        let device = self.device.clone();
        let pass = self.render_passes[0].clone();

        // create pipelines
        let directional_lighting_pipeline = DirectionalLightingSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());
        let ambient_lighting_pipeline = AmbientLightingSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());
        let renderable_pipeline = RenderableDrawSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());
        let terrain_draw_pipeline = TerrainDrawSystemPipeline::create_graphics_pipeline(device.clone(), pass.clone(), viewport.clone());

        // add pipelines
        let mut pipelines = self.pipelines.lock().unwrap();
        pipelines.clear();
        pipelines.insert(TypeId::of::<DirectionalLightingSystemPipeline>(), directional_lighting_pipeline);
        pipelines.insert(TypeId::of::<RenderableDrawSystemPipeline>(), renderable_pipeline);
        pipelines.insert(TypeId::of::<AmbientLightingSystemPipeline>(), ambient_lighting_pipeline);
        pipelines.insert(TypeId::of::<TerrainDrawSystemPipeline>(), terrain_draw_pipeline);

    }

    fn scale_image_views_to_images(&self, images: &[Arc<SwapchainImage>]){
        // should probably comment some of this i guess
        let image = images[0].clone();
        let dimensions = image.clone().dimensions().width_height();
        if self.diffuse_buffer().image().dimensions().width_height() != dimensions {
            log::debug!("Resizing FrameBuffers");
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

    pub fn get_framebuffer(&self, image_index: u32) -> Arc<Framebuffer> {
        // todo : is there only one image???
        (*self.framebuffers.clone().lock().unwrap())[image_index as usize].clone()
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