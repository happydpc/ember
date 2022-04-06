use crate::core::systems::render_systems::DirectionalLightingSystem;
use crate::core::systems::RequiresGraphicsPipeline;

use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::RenderPass;
use vulkano::format::Format;
use vulkano::swapchain::Swapchain;
use vulkano::device::Device;

use winit::window::Window;

use std::sync::Arc;
use std::any::TypeId;
use std::collections::HashMap;

pub struct SceneState{
    pub pipelines: HashMap<TypeId, Arc<GraphicsPipeline>>,
    pub render_passes: Vec<Arc<RenderPass>>,
}

impl SceneState{
    pub fn new() -> Self {
        SceneState{
            pipelines: HashMap::new(),
            render_passes: Vec::new(),
        }
    }

    pub fn initialize(
        &mut self,
        swapchain: Arc<Swapchain<winit::window::Window>>,
        device: Arc<Device>,
    ){
        let pass = self.create_render_pass(swapchain.format(), device.clone());
        let directional_lighting_pipeline = DirectionalLightingSystem::create_graphics_pipeline(device.clone(), pass.clone());

        self.render_passes.push(pass);
        self.pipelines.insert(TypeId::of::<DirectionalLightingSystem>(), directional_lighting_pipeline);
    }

    fn create_render_pass(&self, swapchain_format: Format, device: Arc<Device>) -> Arc<RenderPass> {

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
        render_pass
    }

    pub fn get_pipeline_for_system<S: 'static>(&self) -> Option<&Arc<GraphicsPipeline>>{
        match self.pipelines.get(&TypeId::of::<S>()) {
            Some(pipeline) => Some(pipeline),
            None => None
        }
    }

   
}