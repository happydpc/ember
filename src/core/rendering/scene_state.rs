use crate::core::systems::render_systems::DirectionalLightingSystem;
use crate::core::systems::render_systems::AmbientLightingSystem;
use crate::core::systems::render_systems::RenderableDrawSystem;
use crate::core::systems::terrain_systems::TerrainDrawSystem;
use crate::core::systems::RequiresGraphicsPipeline;

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
    pub diffuse_buffer: Option<Arc<Mutex<Arc<ImageView<AttachmentImage>>>>>,
    pub normals_buffer: Option<Arc<Mutex<Arc<ImageView<AttachmentImage>>>>>,
    pub depth_buffer: Option<Arc<Mutex<Arc<ImageView<AttachmentImage>>>>>,
    pub viewport: Option<Arc<Mutex<Viewport>>>,
    pub framebuffers: Arc<Mutex<Option<Arc<Framebuffer>>>>,
}

impl SceneState{
    pub fn new() -> Self {
        SceneState{
            pipelines: Mutex::new(HashMap::new()),
            render_passes: Vec::new(),
            diffuse_buffer: None,
            normals_buffer: None,
            depth_buffer: None,
            viewport: None,
            framebuffers: Arc::new(Mutex::new(None)),
        }
    }

    pub fn initialize(
        &mut self,
        swapchain: Arc<Swapchain<winit::window::Window>>,
        device: Arc<Device>,
    ){  
        // crucially, this does not initialize the framebuffer. to initialize the framebuffer, we must call scale framebuffers to images

        // create buffers
        let (
            diffuse_buffer,
            normals_buffer,
            depth_buffer
        ) = self.build_buffers(device.clone(), None);
    
        // create pass
        let pass = self.build_render_pass(swapchain.image_format(), device.clone());

        // create pipelines
        let directional_lighting_pipeline = DirectionalLightingSystem::create_graphics_pipeline(device.clone(), pass.clone());
        let ambient_lighting_pipeline = AmbientLightingSystem::create_graphics_pipeline(device.clone(), pass.clone());
        let renderable_pipeline = RenderableDrawSystem::create_graphics_pipeline(device.clone(), pass.clone());
        let terrain_draw_pipeline = TerrainDrawSystem::create_graphics_pipeline(device.clone(), pass.clone());
        
        // create viewport
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        // add passes
        self.render_passes.push(pass);
        
        // add pipelines
        let pipelines = &mut *self.pipelines.lock().unwrap();
        pipelines.insert(TypeId::of::<DirectionalLightingSystem>(), directional_lighting_pipeline);
        pipelines.insert(TypeId::of::<RenderableDrawSystem>(), renderable_pipeline);
        pipelines.insert(TypeId::of::<AmbientLightingSystem>(), ambient_lighting_pipeline);
        pipelines.insert(TypeId::of::<TerrainDrawSystem>(), terrain_draw_pipeline);
        
        // add buffers
        self.diffuse_buffer = Some(Arc::new(Mutex::new(diffuse_buffer)));
        self.normals_buffer = Some(Arc::new(Mutex::new(normals_buffer)));
        self.depth_buffer = Some(Arc::new(Mutex::new(depth_buffer)));

        // add viewport
        self.viewport = Some(Arc::new(Mutex::new(viewport)));
    }

    fn build_render_pass(&self, swapchain_format: Format, device: Arc<Device>) -> Arc<RenderPass> {
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

    fn build_buffers(&self, device: Arc<Device>, image: Option<Arc<ImageView<SwapchainImage<winit::window::Window>>>>)
    -> (Arc<ImageView<AttachmentImage>>, Arc<ImageView<AttachmentImage>>, Arc<ImageView<AttachmentImage>>){
        // For now we create three temporary images with a dimension of 1 by 1 pixel.
        // These images will be replaced the first time we call `frame()`.
        // TODO: use shortcut provided in vulkano 0.6
        let image_dim = {
            match image{
                Some(image) => image.image().dimensions().width_height(),
                None => [1, 1]
            }
        };
        let atch_usage = ImageUsage {
            transient_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };
        let diffuse_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                device.clone(),
                image_dim,
                Format::A2B10G10R10_UNORM_PACK32,
                atch_usage,
            )
            .unwrap(),
        )
        .unwrap();
        let normals_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                device.clone(),
                image_dim,
                Format::R16G16B16A16_SFLOAT,
                atch_usage,
            )
            .unwrap(),
        )
        .unwrap();
        let depth_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                device.clone(),
                image_dim,
                Format::D16_UNORM,
                atch_usage,
            )
            .unwrap(),
        )
        .unwrap();
        (diffuse_buffer, normals_buffer, depth_buffer)
    }

    pub fn scale_scene_state_to_images(&self, image: Arc<ImageView<SwapchainImage<winit::window::Window>>>, device: Arc<Device>){
        self.scale_framebuffers_to_images(image.clone(), device.clone());
        self.rescale_viewport(image.clone());
    }

    fn rescale_viewport(&self, image: Arc<ImageView<SwapchainImage<winit::window::Window>>>){
        let dimensions = image.image().dimensions().width_height();
        match &self.viewport{
            Some(viewport) => viewport.lock().unwrap().dimensions = [dimensions[0] as f32, dimensions[1] as f32],
            None => ()
        }
    }

    fn scale_framebuffers_to_images(&self, image: Arc<ImageView<SwapchainImage<winit::window::Window>>>, device: Arc<Device>){
        // create buffers
        let (
            _diffuse_buffer,
            _normals_buffer,
            _depth_buffer
        ) = self.build_buffers(device.clone(), Some(image.clone()));

        let dimensions = image.clone().image().dimensions().width_height();

        let atch_usage = ImageUsage {
            transient_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };

        let diffuse_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                device.clone(),
                dimensions,
                Format::A2B10G10R10_UNORM_PACK32,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        
        let normals_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                device.clone(),
                dimensions,
                Format::R16G16B16A16_SFLOAT,
                atch_usage,
            ).unwrap(),
        ).unwrap();
        
        let depth_buffer = ImageView::new_default(
            AttachmentImage::with_usage(
                device.clone(),
                dimensions,
                Format::D16_UNORM,
                atch_usage,
            ).unwrap(),
        ).unwrap();

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
        
        *self.framebuffers.clone().lock().unwrap() = Some(framebuffer);
        *self.diffuse_buffer.clone().unwrap().lock().unwrap() = diffuse_buffer;
        *self.normals_buffer.clone().unwrap().lock().unwrap() = normals_buffer;
        *self.depth_buffer.clone().unwrap().lock().unwrap() = depth_buffer;
        
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

    pub fn get_framebuffer_image(&self, _image_num: usize) -> Arc<Framebuffer> {
        self.framebuffers.clone().lock().unwrap().clone().unwrap()
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport.clone().unwrap().lock().unwrap().clone()
    }

    pub fn diffuse_buffer(&self) -> Arc<ImageView<AttachmentImage>> {
        self.diffuse_buffer.clone().unwrap().lock().unwrap().clone()
    }

    pub fn normals_buffer(&self) -> Arc<ImageView<AttachmentImage>> {
        self.normals_buffer.clone().unwrap().lock().unwrap().clone()
    }

    pub fn depth_buffer(&self) -> Arc<ImageView<AttachmentImage>> {
        self.depth_buffer.clone().unwrap().lock().unwrap().clone()
    }
   
}