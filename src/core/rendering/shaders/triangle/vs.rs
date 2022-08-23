vulkano_shaders::shader! {
    ty: "vertex",
    types_meta: {
        use bytemuck::{Pod, Zeroable};

        #[derive(Clone, Copy, Zeroable, Pod)]
    },
    src: "
        #version 450
        layout(location = 0) in vec3 position;
        layout(location = 0) out vec3 outPos;

        layout(set = 0, binding = 0) uniform Data {
            mat4 world;
            mat4 view;
            mat4 proj;
        } uniforms;

        void main() {
            outPos = position;
            // gl_Position = uniforms.mwv * vec4(position, 1.0);
            gl_Position = uniforms.proj * uniforms.view * uniforms.world * vec4(position, 1.0);
            // gl_Position = vec4(position, 1.0) * uniforms.world * uniforms.view * uniforms.proj;
            gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
            gl_Position.y = -gl_Position.y;
        }
    "
}
/*
pub static VERTEX_SHADER_SRC: &'static str = r#"
    #version 140

    in vec3 position;

    void main() {
        gl_Position = vec4(position, 1.0);
    }
"#;
*/
