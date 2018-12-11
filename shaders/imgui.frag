#version 450 core

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 color;

uniform sampler2D u_font;

out vec4 out_color;

void main() {
    // out_color = color * texture(u_font, uv.st);
    out_color = color;
}
