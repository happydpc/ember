pub static VERTEX_SHADER_SRC: &'static str = r#"
    #version 140

    in vec3 position;

    void main() {
        gl_Position = vec4(position, 1.0);
    }
"#;
