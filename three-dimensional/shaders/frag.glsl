#version 450

layout(location = 0) in vec3 color_out;

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(color_out, 1.0);
}
