#version 450 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_color;

uniform mat4 u_proj;


void main() {
    out_uv = uv;
    out_color = color;

    gl_Position = u_proj * vec4(position.xy,0.0,1.0);

}