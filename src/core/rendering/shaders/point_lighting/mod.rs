/// The vertex shader
pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/core/rendering/shaders/point_lighting/vert.glsl",
    }
}

/// The fragment shader
pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/core/rendering/shaders/point_lighting/frag.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}