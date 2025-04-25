#version 450

layout(location = 0) in vec2 uv_out;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform Data {
    float offset;
} uniforms;

void main() {
	float demand = 0.5 * pow(uv_out[0], 2.0);
	float supply = uniforms.offset + 1.0 - pow(uv_out[0], 2.0);
	if (abs(demand - uv_out[1]) < 0.01 || abs(supply - uv_out[1]) < 0.01) {
		f_color = vec4(0.0, 0.0, 0.0, 1.0);
	} else if (demand > supply && uv_out[1] > supply && uv_out[1] < demand) {
		f_color = vec4(0.5, 1.0, 0.5, 1.0);
	} else {
		discard;
	}
}
