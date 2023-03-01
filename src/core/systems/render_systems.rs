use std::sync::Arc;
use std::convert::TryInto;


use bevy_ecs::prelude::{
    Query,
    Res,
    ResMut,
    With,
};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::{
    allocator::{StandardDescriptorSetAllocator},
    PersistentDescriptorSet,
    WriteDescriptorSet,
};
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};

use crate::core::plugins::components::{
    RenderableComponent,
    CameraComponent,
    CameraMatrices,
    TransformComponent,
    DirectionalLightComponent,
    AmbientLightingComponent,
    GeometryComponent,
};
use crate::core::rendering::geometries::geometry_primitives::{
    Vertex,
};
use crate::core::rendering::shaders;


use crate::core::managers::render_manager::{
    TriangleSecondaryBuffers,
    LightingSecondaryBuffers,
};
use crate::core::managers::input_manager::KeyInputQueue;
use crate::core::rendering::SceneState;

use ember_math::Matrix4f;



use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::StateMode;

use vulkano::pipeline::PartialStateMode;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use vulkano::pipeline::graphics::viewport::{ViewportState, Viewport};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::rasterization::{RasterizationState, CullMode, FrontFace};
use vulkano::pipeline::graphics::color_blend::{
    BlendFactor, BlendOp, AttachmentBlend, ColorBlendState,
};

use vulkano::render_pass::Subpass;
use vulkano::render_pass::RenderPass;
use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::TypedBufferAccess;

use vulkano::command_buffer::{
    CommandBufferUsage,
    CommandBufferInheritanceInfo,
};
use vulkano::command_buffer::AutoCommandBufferBuilder;

use winit::window::Window;
use winit::event::ModifiersState;
use winit::event::VirtualKeyCode;

use crate::core::managers::render_manager::VulkanAllocators;
use crate::core::managers::render_manager::{
    QueueResource,
    DeviceResource,
    SurfaceResource,
    SceneStateResource,
};


use log;

pub trait RequiresGraphicsPipeline{
    fn create_graphics_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>, viewport: Viewport) -> Arc<GraphicsPipeline>;
}


pub fn RenderableInitializerSystem(
    mut query: Query<&mut RenderableComponent>,
    device: Res<DeviceResource>,
){
    log::debug!("Running renderable init system...");
    for mut renderable in query.iter_mut() {
        // if renderable.initialized == false{
            log::debug!("Init renderable.");
            renderable.initialize(device.0.clone());
        // }
    }
}

pub fn CameraUpdateSystem(
    mut query: Query<(&mut CameraComponent, &mut TransformComponent)>,
    surface: Res<SurfaceResource>,
    mut state: ResMut<CameraMatrices>,
){
    log::debug!("Running camera update system...");
    let binding = surface.0.clone();
    let window = binding.object().unwrap().downcast_ref::<Window>().unwrap();
    let dimensions: [u32; 2] = window.inner_size().into();
    let aspect = dimensions[0] as f32/ dimensions[1] as f32;
    for (mut camera, _transform) in query.iter_mut(){
        log::debug!("updating camera");
        camera.aspect = aspect;
        camera.calculate_view();
        camera.calculate_perspective();
        state.view = camera.get_view();
        state.perspective = camera.get_perspective();
    }
}

pub struct RenderableDrawSystemPipeline;
impl RequiresGraphicsPipeline for RenderableDrawSystemPipeline{
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


pub fn RenderableDrawSystem(
    query: Query<(&TransformComponent, &GeometryComponent, With<RenderableComponent>)>,
    camera_state: Res<CameraMatrices>,
    queue_res: Res<QueueResource>,
    scene_state_res: Res<SceneStateResource>,
    allocators: Res<VulkanAllocators>,
    mut buffer_vec: ResMut<TriangleSecondaryBuffers>,
){
    log::debug!("Running RenderableDrawSystem...");
    let queue = queue_res.0.clone();
    let scene_state = scene_state_res.0.clone();
    let viewport = scene_state.viewport();
    let pipeline: Arc<GraphicsPipeline> = scene_state.get_pipeline_for_system::<RenderableDrawSystemPipeline>().expect("Could not get pipeline from scene_state.");
    let pass = scene_state.diffuse_pass.clone();

    let layout = pipeline.layout().set_layouts().get(0).unwrap();
    for (transform, geometry, _has_renderable) in query.iter() {
        log::debug!("Creating secondary command buffer builder...");
        // create buffer buildres
        // create a command buffer builder
        let mut builder = AutoCommandBufferBuilder::secondary(
            &allocators.command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(pass.clone().into()),
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

        let uniform_buffer_subbuffer = {
            // create matrix
            let translation_matrix: Matrix4f = Matrix4f::from_translation(transform.global_position());
            let rotation_matrix: Matrix4f = transform.rotation();
            let scale_matrix: Matrix4f = Matrix4f::from_scale(transform.scale());
            let model_to_world: Matrix4f = translation_matrix * rotation_matrix * scale_matrix;

            // state is view, perspective
            // TODO : de-couple model matrix and camera matrices            
            let uniform_buffer_data = shaders::triangle::vs::ty::Data{
                world: model_to_world.transpose().into(),
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

        log::debug!("Building secondary commands for renderable draw...");
        let _ = &builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                set.clone(),
            )
            .bind_vertex_buffers(0, geometry.vertex_buffer().clone())
            .bind_index_buffer(geometry.index_buffer().clone())
            .draw_indexed(
                (*geometry.index_buffer()).len() as u32,
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

pub struct DirectionalLightingSystemPipeline;
impl RequiresGraphicsPipeline for DirectionalLightingSystemPipeline{
    fn create_graphics_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>, viewport: Viewport) -> Arc<GraphicsPipeline>{

        let vs = shaders::directional_lighting::vs::load(device.clone()).expect("failed to create vertex shader for direcitonal lighting system.");
        let fs = shaders::directional_lighting::fs::load(device.clone()).expect("failed to create fragment shader for directional lighting system.");

        GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .color_blend_state(ColorBlendState::new(Subpass::from(render_pass.clone(), 1).unwrap().num_color_attachments()).blend(
                AttachmentBlend {
                    color_op: BlendOp::Add,
                    color_source: BlendFactor::One,
                    color_destination: BlendFactor::One,
                    alpha_op: BlendOp::Max,
                    alpha_source: BlendFactor::One,
                    alpha_destination: BlendFactor::One,
                },
            ))
            .render_pass(Subpass::from(render_pass.clone(), 1).unwrap())
            .build(device.clone())
            .unwrap()
    }
}


pub fn DirectionalLightingSystem(
    query: Query<&DirectionalLightComponent>,
    queue_res: Res<QueueResource>,
    scene_state_res: Res<SceneStateResource>,
    allocators: Res<VulkanAllocators>,
    mut buffer_vec: ResMut<LightingSecondaryBuffers>,
){
    log::debug!("Running Directional Lighting System...");
    let queue = queue_res.0.clone();
    let scene_state = scene_state_res.0.clone();

    // v buffer
    let vertex_buffer = {
        CpuAccessibleBuffer::from_iter(
            &allocators.memory_allocator.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            [
                Vertex {
                    position: [-1.0, -1.0, 0.0],
                },
                Vertex {
                    position: [1.0, -1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                },
                Vertex {
                    position: [-1.0, -1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0, 0.0],
                },
            ].iter().cloned()
        )
        .expect("failed to create buffer")
    };
    let color_input = scene_state.diffuse_buffer();
    let normals_input = scene_state.normals_buffer();
    let viewport = scene_state.viewport();
    let pipeline: Arc<GraphicsPipeline> = scene_state.get_pipeline_for_system::<DirectionalLightingSystemPipeline>().expect("Could not get pipeline from scene_state.");
    let subpass = scene_state.lighting_pass.clone();
    let layout = pipeline.layout().set_layouts().get(0).expect("Couldn't get pipeline layout.");

    for light_comp in query.iter(){
        let push_constants = shaders::directional_lighting::fs::ty::PushConstants {
            color: [light_comp.color.x, light_comp.color.y, light_comp.color.z, 1.0],
            direction: light_comp.direction.extend(0.0).into(),
        };

        let descriptor_set = PersistentDescriptorSet::new(
            &allocators.descriptor_set_allocator.clone(),
            layout.clone(),
            [
                WriteDescriptorSet::image_view(0, color_input.clone()),
                WriteDescriptorSet::image_view(1, normals_input.clone()),
            ],
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::secondary(
            &allocators.command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(subpass.clone().into()),
                ..Default::default()
            },
        ).unwrap();

        builder
            .bind_pipeline_graphics(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.clone().layout().clone(),
                0,
                descriptor_set.clone(),
            )
            .push_constants(
                pipeline.layout().clone(),
                0,
                push_constants
            )
            .bind_vertex_buffers(
                0,
                vertex_buffer.clone(),
            )
            .draw(
                vertex_buffer.len().try_into().unwrap(),
                1,
                0,
                0
            )
            .unwrap();

        // build and push 
        let command_buffer = builder.build().expect("Failed to build secondary command buffer.");
        buffer_vec.buffers.push(Box::new(command_buffer));
    }
}

pub struct AmbientLightingSystemPipeline;
impl RequiresGraphicsPipeline for AmbientLightingSystemPipeline{
    fn create_graphics_pipeline(device: Arc<Device>, render_pass: Arc<RenderPass>, viewport: Viewport) -> Arc<GraphicsPipeline>{

        let vs = shaders::ambient_lighting::vs::load(device.clone()).expect("failed to create vertex shader for ambient lighting system.");
        let fs = shaders::ambient_lighting::fs::load(device.clone()).expect("failed to create fragment shader for ambient lighting system.");

        GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .color_blend_state(ColorBlendState::new(Subpass::from(render_pass.clone(), 1).unwrap().num_color_attachments()).blend(
            AttachmentBlend {
                color_op: BlendOp::Add,
                color_source: BlendFactor::One,
                color_destination: BlendFactor::One,
                alpha_op: BlendOp::Max,
                alpha_source: BlendFactor::One,
                alpha_destination: BlendFactor::One,
            },
        ))
        .render_pass(Subpass::from(render_pass.clone(), 1).unwrap())
        .build(device.clone())
        .unwrap()
    }
}


pub fn AmbientLightingSystem(
    query: Query<&AmbientLightingComponent>,
    queue_res: Res<QueueResource>,
    scene_state_res: Res<SceneStateResource>,
    allocators: Res<VulkanAllocators>,
    mut buffer_vec: ResMut<LightingSecondaryBuffers>,
){
    log::debug!("Running ambient Lighting System...");
    let queue = queue_res.0.clone();
    let scene_state = scene_state_res.0.clone();

    // v buffer
    let vertex_buffer = {
        CpuAccessibleBuffer::from_iter(
            &allocators.memory_allocator.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            [
                Vertex {
                    position: [-1.0, -1.0, 0.0],
                },
                Vertex {
                    position: [1.0, -1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                },
                Vertex {
                    position: [-1.0, -1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0, 0.0],
                },
            ]
            .iter()
            .cloned(),
        )
        .expect("failed to create buffer")
    };
    let color_input = scene_state.diffuse_buffer();
    let viewport = scene_state.viewport();
    let pipeline: Arc<GraphicsPipeline> = scene_state.get_pipeline_for_system::<AmbientLightingSystemPipeline>().expect("Could not get pipeline from scene_state.");
    let subpass = scene_state.lighting_pass.clone();
    let layout = pipeline.layout().set_layouts().get(0).expect("Couldn't get pipeline layout.");

    for light_comp in query.iter(){
        let push_constants = shaders::ambient_lighting::fs::ty::PushConstants {
            color: [light_comp.color.x, light_comp.color.y, light_comp.color.z, 1.0],
        };

        let descriptor_set = PersistentDescriptorSet::new(
            &allocators.descriptor_set_allocator.clone(),
            layout.clone(),
            [
                WriteDescriptorSet::image_view(0, color_input.clone()),
            ]
        ).unwrap();

        let mut builder = AutoCommandBufferBuilder::secondary(
            &allocators.command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(subpass.clone().into()),
                ..Default::default()
            },
        ).unwrap();

        builder
            .bind_pipeline_graphics(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.clone().layout().clone(),
                0,
                descriptor_set.clone(),
            )
            .push_constants(
                pipeline.layout().clone(),
                0,
                push_constants
            )
            .bind_vertex_buffers(
                0,
                vertex_buffer.clone(),
            )
            .draw(
                vertex_buffer.len().try_into().unwrap(),
                1,
                0,
                0
            )
            .unwrap();

        // build and push 
        let command_buffer = builder.build().expect("Failed to build secondary command buffer.");
        buffer_vec.buffers.push(Box::new(command_buffer));
    }
}


pub fn RenderableAssemblyStateModifierSystem(
    scene_state_res: Res<SceneStateResource>,
    read_input: Res<KeyInputQueue>,
    device_res: Res<DeviceResource>
){
    log::debug!("Renderable wireframe sysetm...");
    let input = read_input.queue.clone();
    let scene_state = scene_state_res.0.clone();
    let device = device_res.0.clone();
    let modifiers = read_input.modifiers_state.clone();
    if modifiers.shift() && modifiers.alt() && input.contains(&VirtualKeyCode::Z){
        let topology = match scene_state
            .get_pipeline_for_system::<RenderableDrawSystemPipeline>()
            .expect("Couldn't get pipeline for renderable draw in wireframe system.")
            .input_assembly_state()
            .topology
        {
            PartialStateMode::Fixed(PrimitiveTopology::TriangleList) => PrimitiveTopology::LineStrip,
            PartialStateMode::Fixed(PrimitiveTopology::LineStrip) => PrimitiveTopology::TriangleList,
            _ => unreachable!(),
        };
        let subpass = scene_state.diffuse_pass.clone();
        let pipeline = RenderableAssemblyStateModifierSystemPipeline::create_renderable_pipeline(
            device.clone(),
            subpass,
            topology, 
            scene_state.viewport()
        );
        scene_state.set_pipeline_for_system::<RenderableDrawSystemPipeline>(pipeline);
    }
}

pub struct RenderableAssemblyStateModifierSystemPipeline;
impl RenderableAssemblyStateModifierSystemPipeline {
    pub fn create_renderable_pipeline(device: Arc<Device>, subpass: Subpass, topology: PrimitiveTopology, viewport: Viewport) -> Arc<GraphicsPipeline> {
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
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
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