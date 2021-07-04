vulkano_shaders::shader! {
    ty: "fragment",
    src: "
        #version 450
        layout(location = 0) out vec4 f_color;
        void main() {
            f_color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "
}
/*
pub static FRAGMENT_SHADER_SRC: &'static str = r#"
    #version 140

    out vec4 color;

    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;
*/
