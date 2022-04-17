use specs::{Join, System, WriteStorage, ReadStorage, ReadExpect, WriteExpect};

use cgmath::Matrix4;

use crate::core::plugins::components::TerrainComponent;
use crate::core::plugins::components::TransformComponent;
use crate::core::systems::RequiresGraphicsPipeline;
use crate::core::rendering::shaders;
use crate::core::rendering::geometries::Vertex;
use crate::core::rendering::render_manager::TriangleSecondaryBuffers;
use crate::core::rendering::SceneState;
use crate::core::systems::render_systems::CameraState;

use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::render_pass::RenderPass;
use vulkano::render_pass::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::rasterization::{RasterizationState, CullMode, FrontFace};
use vulkano::pipeline::StateMode;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::PipelineBindPoint;


use std::sync::{Arc, Mutex};

pub struct TerrainInitSystem;
impl<'a> System<'a> for TerrainInitSystem{
    type SystemData = (
        WriteStorage<'a, TerrainComponent>,
        ReadExpect<'a, Arc<Device>>
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let (mut terrains, device) = data;
        for mut terrain in (&mut terrains).join() {
            {
                terrain.geometry.lock().unwrap().generate_terrain();
            }
            terrain.initialize(device.clone());
        }
    }
}

pub struct TerrainDrawSystem;
impl RequiresGraphicsPipeline for TerrainDrawSystem{
    fn create_graphics_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>) -> Arc<GraphicsPipeline>{

            // compile our shaders
            let vs = shaders::triangle::vs::load(device.clone()).expect("Failed to create vertex shader for triangle draw system.");
            let fs = shaders::triangle::fs::load(device.clone()).expect("Failed to create fragment shader for triangle draw system.");

            let rs = RasterizationState{
                cull_mode: StateMode::Fixed(CullMode::Back),
                front_face: StateMode::Fixed(FrontFace::CounterClockwise),
                ..Default::default()
            };

            let input_assembly_state = InputAssemblyState::new().topology(PrimitiveTopology::LineStrip);

            // create our pipeline. like an opengl program but more specific
            let pipeline = GraphicsPipeline::start()
                // We need to indicate the layout of the vertices.
                .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
                // A Vulkan shader can in theory contain multiple entry points, so we have to specify
                // which one.
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                // The content of the vertex buffer describes a list of triangles.
                .input_assembly_state(input_assembly_state)
                // Use a resizable viewport set to draw over the entire window
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                // See `vertex_shader`.
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .depth_stencil_state(DepthStencilState::simple_depth_test())
                .rasterization_state(rs)
                // We have to indicate which subpass of which render pass this pipeline is going to be used
                // in. The pipeline will only be usable from this particular subpass.
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
                .build(device.clone())
                .expect("Can't build pipeline for renderable draw system.");
            pipeline
    }
}

impl <'a> System<'a> for TerrainDrawSystem{
    type SystemData = (
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, TerrainComponent>,
        ReadExpect<'a, CameraState>,
        ReadExpect<'a, Arc<Queue>>,
        WriteExpect<'a, TriangleSecondaryBuffers>,
        ReadExpect<'a, Arc<SceneState>>,
    );

    fn run(&mut self, data: Self::SystemData){
        log::debug!("Running RenderableDrawSystem...");
        let (transforms,
            terrains,
            camera_state,
            queue,
            mut buffer_vec,
            scene_state,
        ) = data;

        let scene_state: &SceneState = &*scene_state;
        let viewport = scene_state.viewport();
        let pipeline: Arc<GraphicsPipeline> = scene_state.get_pipeline_for_system::<Self>().expect("Could not get pipeline from scene_state.");

        let layout = &*pipeline.layout().set_layouts().get(0).unwrap();
        for (terrain, transform) in (&terrains, &transforms).join() {
            log::debug!("Creating secondary command buffer builder...");
            // create buffer buildres
            // create a command buffer builder
            let mut builder = AutoCommandBufferBuilder::secondary_graphics(
                queue.device().clone(),
                queue.family(),
                CommandBufferUsage::OneTimeSubmit,
                pipeline.subpass().clone(),
            )
            .unwrap();
            
            log::debug!("Binding pipeline graphics for secondary command buffer....");
            // this is the default color of the framebuffer
            builder
                .set_viewport(0, [viewport.clone()])
                .bind_pipeline_graphics(pipeline.clone());

            let uniform_buffer: CpuBufferPool::<shaders::triangle::vs::ty::Data> = CpuBufferPool::new(
                queue.device().clone(),
                BufferUsage::all()
            );

            let g_arc = &terrain.geometry.clone();
            let geometry = g_arc.lock().unwrap();
            let uniform_buffer_subbuffer = {
                // create matrix
                let translation_matrix: Matrix4<f32> = Matrix4::from_translation(transform.global_position);
                let rotation_matrix: Matrix4<f32> = transform.rotation;
                let model_to_world: Matrix4<f32> = rotation_matrix * translation_matrix;

                
                let uniform_buffer_data = shaders::triangle::vs::ty::Data{
                    mwv: (camera_state[1] * camera_state[0] * model_to_world).into()
                };
                uniform_buffer.next(uniform_buffer_data).unwrap()
            };
    
            let set = PersistentDescriptorSet::new(
                layout.clone(),
                [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)]
            ).unwrap();

            log::debug!("Building secondary commands...");
            let _ = &builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    set.clone(),
                )
                .bind_vertex_buffers(0, geometry.vertex_buffer.clone().unwrap().clone())
                // .bind_index_buffer(geometry.index_buffer().clone().unwrap().clone())
                // .draw_indexed(
                //     (*geometry.index_buffer()).len() as u32,
                //     1,
                //     0,
                //     0,
                //     0
                // )
                .draw(
                    geometry.vertex_buffer.clone().unwrap().clone().len() as u32,
                    1,
                    0,
                    0
                )
                .unwrap();
            let command_buffer = builder.build().unwrap();
            buffer_vec.buffers.push(Box::new(command_buffer));
        }
    }
}