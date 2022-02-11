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
};

use crate::core::rendering::shaders::triangle::vs;
use crate::core::rendering::render_manager::SwapchainImageNum;

use cgmath::Matrix4;

use vulkano::device::Device;
use vulkano::swapchain::Surface;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::render_pass::Framebuffer;
use vulkano::buffer::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::TypedBufferAccess;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::device::Queue;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::SubpassContents;
use vulkano::command_buffer::SecondaryCommandBuffer;
use vulkano::pipeline::graphics::viewport::Viewport;

use winit::window::Window;


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
        ReadExpect<'a, Vec<Arc<Framebuffer>>>,
        WriteExpect<'a, Vec<Box<SecondaryCommandBuffer>>>
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
            framebuffers,
            mut buffer_vec,
        ) = data;

        let layout = &*pipeline.layout().descriptor_set_layouts().get(0).unwrap();
        
        for (renderable, transform) in (&renderables, &transforms).join() {
            // create buffer buildres
            // create a command buffer builder
            let mut builder = AutoCommandBufferBuilder::secondary_graphics(
                device.clone(),
                queue.family(),
                CommandBufferUsage::OneTimeSubmit,
                pipeline.subpass().clone()
            )
            .unwrap();

            // this is the default color of the framebuffer
            let clear_values: Vec<f32> = vec![0.2, 0.2, 0.2, 1.0].into();
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
            buffer_vec.push(Box::new(command_buffer));
        }
    }
}