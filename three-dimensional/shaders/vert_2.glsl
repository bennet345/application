#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in mat4 transform;
layout(location = 6) in vec3 supply;
layout(location = 7) in vec3 demand;
layout(location = 8) in vec2 equilibrium;

layout(location = 0) out vec2 uv_out;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    uv_out = uv;
    gl_Position = uniforms.proj * worldview * transform * vec4(position, 1.0);
}
