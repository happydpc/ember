pub struct Renderable{
    pub vertex_buffer: CpuAccessibleBuffer<()>,
}

use vulkano::{
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
    },
    device::{
        Device
    }
};
