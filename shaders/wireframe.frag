#version 450 core

layout(location = 0) in vec4 normal;
layout(location = 1) in vec4 tangent;
layout(location = 2) in vec4 bitangent;
layout(location = 3) in vec2 uv;

out vec4 color;

void main() {
    color = vec4(0,0,0,1.0);
}