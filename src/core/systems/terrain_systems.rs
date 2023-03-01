use bevy_ecs::prelude::{
    Query,
    Res,
    ResMut,
    With,
};
use bevy_ecs::event::Events;

use ember_math::Matrix4f;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;

use crate::core::plugins::components::TerrainComponent;
use crate::core::plugins::components::TransformComponent;
use crate::core::systems::RequiresGraphicsPipeline;
use crate::core::rendering::shaders;
use crate::core::rendering::geometries::Vertex;
use crate::core::managers::render_manager::TriangleSecondaryBuffers;
use crate::core::rendering::SceneState;
use crate::core::plugins::components::CameraMatrices;
use crate::core::managers::input_manager::KeyInputQueue;
use crate::core::systems::ui_systems::EguiState;
use crate::core::plugins::components::TerrainUiComponent;

use crate::core::events::terrain_events::TerrainRecalculateEvent;

use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::{CommandBufferUsage, CommandBufferInheritanceInfo};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::descriptor_set::{PersistentDescriptorSet};
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::render_pass::RenderPass;
use vulkano::render_pass::Subpass;
use vulkano::pipeline::PartialStateMode;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::graphics::rasterization::{RasterizationState, CullMode, FrontFace};
use vulkano::pipeline::StateMode;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{ViewportState, Viewport};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::memory::allocator::{StandardMemoryAllocator, MemoryUsage};

use crate::core::managers::render_manager::VulkanAllocators;
use crate::core::managers::render_manager::{
    DeviceResource,
    SurfaceResource,
    QueueResource,
    SceneStateResource,
};

use winit::event::VirtualKeyCode;
use winit::event::ModifiersState;

use std::sync::{Arc};

pub fn TerrainInitSystem(
    mut query: Query<&mut TerrainComponent>,
    allocators: Res<VulkanAllocators>,
){
    log::info!("Terrain init system...");
    for mut terrain in query.iter_mut() {
        {
            log::info!("Generating Terrain Geometry");
            terrain.geometry.lock().unwrap().generate_terrain();
        }
        terrain.initialize(allocators.memory_allocator.clone());
    }
}

pub fn TerrainUpdateSystem(
    mut query: Query<&mut TerrainComponent>,
    mut recalculate_events: ResMut<Events<TerrainRecalculateEvent>>,
    allocators: Res<VulkanAllocators>,
){
    let mut reader = recalculate_events.get_reader();
    for _event in reader.iter(&recalculate_events){
        for mut terrain in query.iter_mut(){
            {
                terrain.geometry.lock().unwrap().generate_terrain();
            }
            terrain.initialize(allocators.memory_allocator.clone());
        }
    }
    recalculate_events.clear();
}

pub struct TerrainDrawSystemPipeline;
impl RequiresGraphicsPipeline for TerrainDrawSystemPipeline{
    fn create_graphics_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>, viewport: Viewport) -> Arc<GraphicsPipeline>{

            // compile our shaders
            let vs = shaders::triangle::vs::load(device.clone()).expect("Failed to create vertex shader for triangle draw system.");
            let fs = shaders::triangle::fs::load(device.clone()).expect("Failed to create fragment shader for triangle draw system.");

            let rs = RasterizationState{
                cull_mode: StateMode::Fixed(CullMode::Back),
                front_face: StateMode::Fixed(FrontFace::CounterClockwise),
                ..Default::default()
            };

            let input_assembly_state = InputAssemblyState::new().topology(PrimitiveTopology::TriangleList);

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
                .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
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


pub fn TerrainDrawSystem(
    query: Query<(&TransformComponent, &TerrainComponent)>,
    camera_state: Res<CameraMatrices>,
    queue_res: Res<QueueResource>,
    scene_state_res: Res<SceneStateResource>,
    allocators: Res<VulkanAllocators>,
    mut buffer_vec: ResMut<TriangleSecondaryBuffers>,
){
    log::debug!("Running Terrain Draw System...");
    let queue = queue_res.0.clone();
    let scene_state = scene_state_res.0.clone();
    let viewport = scene_state.viewport();
    let pipeline: Arc<GraphicsPipeline> = scene_state.get_pipeline_for_system::<TerrainDrawSystemPipeline>().expect("Could not get pipeline from scene_state.");
    let subpass =  scene_state.diffuse_pass.clone();

    let layout = pipeline.layout().set_layouts().get(0).unwrap();
    for (transform, terrain) in query.iter() {
        log::debug!("Creating secondary command buffer builder...");
        // create buffer builders
        // create a command buffer builder
        let mut builder = AutoCommandBufferBuilder::secondary(
            &allocators.command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(subpass.clone().into()),
                ..Default::default()
            },
        ).unwrap();
        
        log::debug!("Binding pipeline graphics for secondary command buffer....");
        // this is the default color of the framebuffer
        builder
            .bind_pipeline_graphics(pipeline.clone());


        let uniform_buffer = CpuBufferPool::<shaders::triangle::vs::ty::Data>::new(
            allocators.memory_allocator.clone(),
            BufferUsage {
                uniform_buffer: true,
                ..BufferUsage::empty()
            },
            MemoryUsage::Upload,
        );

        let g_arc = &terrain.geometry.clone();
        let geometry = g_arc.lock().unwrap();
        let uniform_buffer_subbuffer = {
            // create matrix
            let translation_matrix: Matrix4f = Matrix4f::from_translation(transform.global_position());
            let rotation_matrix: Matrix4f = transform.rotation();
            let scale_matrix: Matrix4f = Matrix4f::from_scale(transform.scale());
            let model_to_world: Matrix4f = rotation_matrix * translation_matrix * scale_matrix;

            
            let uniform_buffer_data = shaders::triangle::vs::ty::Data{
                // mwv: (camera_state[1] * camera_state[0] * model_to_world).into()
                // mwv: (camera_state[1] * model_to_world).into()
                world: model_to_world.into(),
                view: camera_state.view.clone().into(),
                proj: camera_state.perspective.clone().into()
            };
            uniform_buffer.from_data(uniform_buffer_data).unwrap()
        };

        let set = PersistentDescriptorSet::new(
            &allocators.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
        )
        .unwrap();

        log::debug!("Building secondary commands...");
        let _ = &builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                set.clone(),
            )
            .bind_vertex_buffers(0, geometry.vertex_buffer.clone().unwrap().clone())
            .bind_index_buffer(geometry.index_buffer.clone().unwrap().clone())
            .draw_indexed(
                (*geometry.index_buffer.clone().unwrap()).len() as u32,
                1,
                0,
                0,
                0
            )
            .unwrap();
        let command_buffer = builder.build().unwrap();
        buffer_vec.buffers.push(Box::new(command_buffer));
    }
}


pub fn TerrainAssemblyStateModifierSystem(
    scene_state_res: Res<SceneStateResource>,
    read_input: Res<KeyInputQueue>,
    device_res: Res<DeviceResource>,
){
    log::debug!("Terrain wireframe system...");
    let input = read_input.queue.clone();
    let scene_state = scene_state_res.0.clone();
    let device = device_res.0.clone();
    let modifiers = input.modifiers_state.clone();
    if modifiers.shift() && modifiers.alt() && input.contains(&VirtualKeyCode::Z){
        let topology = match scene_state
            .get_pipeline_for_system::<TerrainDrawSystemPipeline>()
            .expect("Couldn't get pipeline for renderable draw in wireframe system.")
            .input_assembly_state()
            .topology
        {
            PartialStateMode::Fixed(PrimitiveTopology::TriangleList) => PrimitiveTopology::LineStrip,
            PartialStateMode::Fixed(PrimitiveTopology::LineStrip) => PrimitiveTopology::TriangleList,
            _ => unreachable!(),
        };
        let subpass = Subpass::from(scene_state.render_passes[0].clone(), 0).unwrap();
        let pipeline = TerrainAssemblyStateSystemPipeline.create_pipeline(device.clone(), subpass, topology);
        scene_state.set_pipeline_for_system::<TerrainDrawSystemPipeline>(pipeline);
    }
}

pub struct TerrainAssemblyStateSystemPipeline;
impl TerrainAssemblyStateSystemPipeline {
    pub fn create_pipeline(&self, device: Arc<Device>, subpass: Subpass, topology: PrimitiveTopology) -> Arc<GraphicsPipeline> {
        // compile our shaders
        let vs = shaders::triangle::vs::load(device.clone()).expect("Failed to create vertex shader for triangle draw system.");
        let fs = shaders::triangle::fs::load(device.clone()).expect("Failed to create fragment shader for triangle draw system.");

        let rs = RasterizationState{
            cull_mode: StateMode::Fixed(CullMode::Back),
            front_face: StateMode::Fixed(FrontFace::CounterClockwise),
            ..Default::default()
        };

        let input_assembly_state = InputAssemblyState::new().topology(topology);

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
            .render_pass(subpass)
            // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
            .build(device.clone())
            .expect("Can't build pipeline for renderable draw system.");
        pipeline
    }
}


pub fn TerrainUiSystem(
    mut query: Query<&mut TerrainComponent, With<TerrainUiComponent>>,
    mut terrain_recalc_events: ResMut<Events<TerrainRecalculateEvent>>,
    egui_state: Res<EguiState>,
){
    log::debug!("Terrain ui system...");

    let ctx = egui_state.ctx.clone();
    for terrain in query.iter_mut(){
        let mut size = terrain.get_size();
        let mut amplitude = terrain.get_amplitude();

        egui::Window::new("Terrain Settings")
            .show(&ctx, |ui| {
                ui.horizontal(|ui|{
                    ui.label("Size");
                    ui.add(egui::Slider::new(&mut size, 2..=500).step_by(1.0));
                });
                ui.horizontal(|ui|{
                    ui.label("Amplidutde");
                    ui.add(egui::Slider::new(&mut amplitude, 0.1..=50.0).step_by(0.1));
                });
            });
        if size < 1 {
            size = 1;
        }
        let size_changed = size != terrain.get_size();
        if size_changed {
            terrain.set_size(size as usize);
        }
        let amplitude_changed = amplitude != terrain.get_amplitude();
        if amplitude_changed {
            terrain.set_amplitude(amplitude);
        }

        if amplitude_changed || size_changed {
            terrain_recalc_events.send(TerrainRecalculateEvent{});
        }
        // terrain.set_size(size as usize);
    }
}