#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in mat4 transform;
layout(location = 5) in vec3 color;

layout(location = 0) out vec3 color_out;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    color_out = color;
    gl_Position = uniforms.proj * worldview * transform * vec4(position, 1.0);
}
