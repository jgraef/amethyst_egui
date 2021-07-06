#version 450

#include "header/math.frag"
#include "header/environment.frag"

layout(set = 1, binding = 0) uniform sampler2D albedo;

layout(location = 0) in VertexData {
    vec2 uv;
    vec4 color;
} vertex;

layout(location = 0) out vec4 out_color;


void main() {
    vec4 color = texture(albedo, vertex.uv) * vertex.color;
    if (color.a == 0.0) {
        discard;
    }

    out_color = color;
}
