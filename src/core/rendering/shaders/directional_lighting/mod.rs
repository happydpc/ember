/// The vertex shader
pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/core/rendering/shaders/directional_lighting/vert.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

/// The fragment shader
pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/core/rendering/shaders/directional_lighting/frag.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}