#version 450 core

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 normal;
layout(location = 2) in vec4 tangent;
layout(location = 3) in vec4 bitangent;
layout(location = 4) in vec2 uv;

layout(location = 0) out vec4 out_normal;
layout(location = 1) out vec4 out_tangent;
layout(location = 2) out vec4 out_bitangent;
layout(location = 3) out vec2 out_uv;

uniform mat4 projection;

// FUTURE: https://learnopengl.com/Lighting/Basic-Lighting
void main() {
    out_normal = normal;
    out_tangent = tangent;
    out_bitangent = bitangent;
    out_uv = uv;

    // Output hardware position
    vec4 pos = projection * position;
    gl_Position = pos;

}