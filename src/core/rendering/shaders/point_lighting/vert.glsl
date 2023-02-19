#version 450
layout(location = 0) in vec3 position;
layout(location = 0) out vec2 v_screen_coords;
void main() {
    v_screen_coords = position.xy;
    gl_Position = vec4(position, 1.0);
}