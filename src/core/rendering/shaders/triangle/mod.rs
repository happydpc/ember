// pub mod vs;
// pub mod fs;

/// The shaders used to render the gui

/// The vertex shader
pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/core/rendering/shaders/triangle/vert.glsl",
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
        path: "src/core/rendering/shaders/triangle/frag.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}
