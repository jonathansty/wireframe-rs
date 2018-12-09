#version 450 core

layout(location = 0) in vec4 position;

void main() {
    // Output hardware position
    gl_Position = position;

}