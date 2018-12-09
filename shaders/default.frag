#version 450 core

layout(location = 0) in vec4 normal;
layout(location = 1) in vec4 tangent;
layout(location = 2) in vec4 bitangent;
layout(location = 3) in vec2 uv;

out vec4 color;

vec3 light = vec3(0.33, -0.33, 0.33);
vec3 light_color = vec3(1,1,1);
float calculate_diffuse(vec3 L, vec3 N){
    return dot(N,L);
}

void main() {
    float strength = 0.05;
    vec3 ambient = strength * light_color;

    vec3 L = normalize(light);
    float D = clamp(calculate_diffuse(L, normal.xyz),0.0,1.0);
    vec3 object = vec3(1,1,1);
    color = vec4(ambient*object + D * object, 1);
}
