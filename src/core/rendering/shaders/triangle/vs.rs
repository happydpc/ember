vulkano_shaders::shader! {
    ty: "vertex",
    src: "
        #version 450
        layout(location = 0) in vec2 position;
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
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
