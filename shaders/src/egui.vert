#version 450

#include "header/math.frag"
#include "header/environment.frag"


layout(std140, set = 0, binding = 0) uniform EguiViewArgs {
    uniform vec4 clip_rect;
    uniform mat4 view;
};


layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;

layout(location = 0) out VertexData {
    vec2 uv;
    vec4 color;
} vertex_data;


// See https://github.com/emilk/egui/blob/26d576f5101dfa1219f79bf9c99e29c577487cd3/egui_glium/src/painter.rs#L19.
vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));
}
vec4 linear_from_srgba(vec4 srgba) {
    return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}


void main() {
    vertex_data.uv = uv;
    vertex_data.color = linear_from_srgba(color);
    gl_Position = view * vec4(pos, 0.0, 1.0);
}
