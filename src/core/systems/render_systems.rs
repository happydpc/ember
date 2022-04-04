use std::sync::Arc;
use specs::{
    System,
    SystemData,
    ReadExpect,
    WriteStorage,
    Join,
    WriteExpect,
    ReadStorage,
};
use crate::core::plugins::components::{
    RenderableComponent,
    CameraComponent,
    TransformComponent,
    DirectionalLightComponent,
};
use crate::core::rendering::geometries::Vertex;
use crate::core::rendering::shaders::triangle::vs;
use crate::core::rendering::render_manager::{
    SwapchainImageNum,
    TriangleSecondaryBuffers,
    LightingSecondaryBuffers,
    AmbientLightingPipeline,
    DirectionalLightingPipelne,
    PointLightingPipeline,
};
use crate::core::rendering::shaders::*;
use crate::core::rendering::frame_handler::{DiffuseImage, DepthImage, NormalsImage};

use cgmath::Matrix4;

use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::swapchain::Surface;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::DynamicState;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::Subpass;
use vulkano::render_pass::RenderPass;
use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::TypedBufferAccess;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::SubpassContents;
use vulkano::command_buffer::SecondaryCommandBuffer;


use winit::window::Window;

use log;


pub struct RenderableInitializerSystem;

impl<'a> System<'a> for RenderableInitializerSystem{
    type SystemData = (
        ReadExpect<'a, Arc<Device>>,
        WriteStorage<'a, RenderableComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (device, mut renderable) = data;
        let device = &*device;
        for renderable in (&mut renderable).join() {
            if renderable.initialized() == false{
                log::debug!("Init renderable.");
                renderable.initialize(device.clone());
            }
        }
    }
}

pub type CameraState = [Matrix4<f32>; 2];
pub struct CameraUpdateSystem;

impl<'a> System<'a> for CameraUpdateSystem{
    type SystemData = (
        WriteStorage<'a, CameraComponent>,
        ReadExpect<'a, Arc<Surface<Window>>>,
        WriteExpect<'a, CameraState>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let (mut cams, surface, mut state) = data;
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        let aspect = dimensions[0] as f32/ dimensions[1] as f32;
        for camera in (&mut cams).join(){
            log::debug!("updating camera");
            camera.aspect = aspect;
            camera.calculate_view();

            // somehow make this unique idk
            *state = [camera.get_view(), camera.get_perspective()];
        }
    }
}

pub struct RenderableDrawSystem;

impl <'a> System<'a> for RenderableDrawSystem{
    type SystemData = (
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, RenderableComponent>,
        ReadExpect<'a, Arc<GraphicsPipeline>>,
        ReadExpect<'a, CameraState>,
        ReadExpect<'a, Arc<Device>>,
        ReadExpect<'a, Arc<Queue>>,
        ReadExpect<'a, Viewport>,
        ReadExpect<'a, SwapchainImageNum>,
        ReadExpect<'a, Arc<RenderPass>>,
        WriteExpect<'a, TriangleSecondaryBuffers>
    );

    fn run(&mut self, mut data: Self::SystemData){
        let (mut transforms,
            renderables,
            pipeline,
            camera_state,
            device,
            queue,
            viewport,
            image_num,
            render_pass,
            mut buffer_vec,
        ) = data;

        let layout = &*pipeline.layout().descriptor_set_layouts().get(0).unwrap();
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        
        for (renderable, transform) in (&renderables, &transforms).join() {
            log::debug!("Creating secondary command buffer builder...");
            // create buffer buildres
            // create a command buffer builder
            let mut builder = AutoCommandBufferBuilder::secondary_graphics(
                device.clone(),
                queue.family(),
                CommandBufferUsage::OneTimeSubmit,
                subpass.clone(),
            )
            .unwrap();
            
            log::debug!("Binding pipeline graphics for secondary command buffer....");
            // this is the default color of the framebuffer
            builder
                .set_viewport(0, [viewport.clone()])
                .bind_pipeline_graphics(pipeline.clone());

            // create matrix
            let translation_matrix: Matrix4<f32> = Matrix4::from_translation(transform.global_position);
            let rotation_matrix: Matrix4<f32> = transform.rotation;
            let model_to_world: Matrix4<f32> = rotation_matrix * translation_matrix;

            let g_arc = &renderable.geometry();
            let geometry = g_arc.lock().unwrap();
            let m = vs::ty::Data{
                mwv: (camera_state[1] * camera_state[0] * model_to_world).into()
            };

            let uniform_buffer: CpuBufferPool::<vs::ty::Data> = CpuBufferPool::new(
                device.clone(),
                BufferUsage::all()
            );

            let uniform_buffer_subbuffer = {
                uniform_buffer.next(m).unwrap()
            };
    
            let set = PersistentDescriptorSet::new(
                layout.clone(),
                [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)]
            ).unwrap();

            log::debug!("Building secondary commands...");
            &builder
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
            // builder.end_render_pass().unwrap();
            // actually build command buffer now
            let command_buffer = builder.build().unwrap();
            log::debug!("Pushing secondary command buffer to vec...");
            buffer_vec.push(Box::new(command_buffer));
        }
    }
}

pub struct DirectionalLightingHandler;

impl <'a> System<'a> for DirectionalLightingHandler {
    type SystemData = (
        ReadStorage<'a, DirectionalLightComponent>,
        ReadExpect<'a, Viewport>,
        WriteExpect<'a, DiffuseImage>,
        WriteExpect<'a, NormalsImage>,
        ReadExpect<'a, Arc<DirectionalLightingPipelne>>,
        ReadExpect<'a, Arc<Queue>>,
        ReadExpect<'a, Arc<RenderPass>>,
        WriteExpect<'a, LightingSecondaryBuffers>,
    );

    fn run(&mut self, mut data: Self::SystemData){

        let (
            light_comps,
            viewport,
            mut _color_input,
            mut _normals_input,
            pipeline,
            queue,
            renderpass,
            mut buffer_vec
        ) = data;

        // v buffer
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                queue.device().clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-1.0, -1.0, 0.0],
                    },
                    Vertex {
                        position: [-1.0, 3.0, 0.0],
                    },
                    Vertex {
                        position: [3.0, -1.0, 0.0],
                    },
                ]
                .iter()
                .cloned(),
            )
            .expect("failed to create buffer")
        };

        let color_input = &*_color_input;
        let normals_input = &*_normals_input;

        for light_comp in (&light_comps).join(){

            let push_constants = directional_lighting::fs::ty::PushConstants {
                color: [light_comp.color[0], light_comp.color[1], light_comp.color[2], 1.0],
                direction: light_comp.direction.extend(0.0).into(),
            };

            // let layout = pipeline
            //     .clone()
            //     .layout()
            //     .descriptor_set_layouts()
            //     .get(0)
            //     .unwrap();

            let layout = &*pipeline.layout().descriptor_set_layouts().get(0).unwrap();
            

            let descriptor_set = PersistentDescriptorSet::new(
                layout.clone(),
                [
                    WriteDescriptorSet::image_view(0, color_input.clone()),
                    WriteDescriptorSet::image_view(1, normals_input.clone()),
                ]
            ).unwrap();

            let mut builder = AutoCommandBufferBuilder::secondary_graphics(
                queue.device().clone(),
                queue.family(),
                CommandBufferUsage::OneTimeSubmit,
                Subpass::from(renderpass.clone(), 1).unwrap(),
            )
            .unwrap();

            builder
                .set_viewport(0, [viewport.clone()])
                .bind_pipeline_graphics(pipeline.clone())
                .bind_vertex_buffers(
                    0,
                    vertex_buffer.clone(),
                )
                .push_constants(
                    // *pipeline.clone().layout(),
                    pipeline.layout().clone(),
                    0,
                    push_constants
                )
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.clone().layout().clone(),
                    0,
                    descriptor_set.clone(),
                )
                .draw(
                    3,
                    1,
                    0,
                    0
                )
                .unwrap();

            // build and push 
            let command_buffer = builder.build().unwrap();
            buffer_vec.push(Box::new(command_buffer));
        }
    }
}
