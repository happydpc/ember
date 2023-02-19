#version 450
layout(location = 0) in vec3 in_pos;

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec3 f_normal;

vec3 calculateScreenSpaceNormal(vec3 p) {
    vec3 dx = dFdx(p);
    vec3 dy = -dFdy(p); // not sure if negation is needed for Vulkan
    return normalize(cross(dx, dy));
}

void main() {
    if (in_pos.z > 1.0) {
        f_color = vec4(0.8, 0.4, 0.4, 1.0);
    }else{
        f_color = vec4(0.5, 0.2, 0.2, 1.0);
    }
    // f_normal = vec3(1.0, 1.0, 1.0);
    f_normal = calculateScreenSpaceNormal(in_pos);
}