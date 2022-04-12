vulkano_shaders::shader! {
    ty: "vertex",
    src: "
        #version 450
        layout(location = 0) in vec3 position;
        layout(location = 0) out vec3 outPos;

        layout(set = 0, binding = 0) uniform Data {
            mat4 mwv;
        } uniforms;

        void main() {
            outPos = position;
            gl_Position = uniforms.mwv * vec4(position, 1.0);
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
