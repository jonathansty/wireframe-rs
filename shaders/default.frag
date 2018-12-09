#version 450 core

layout(location = 0) in vec4 normal;
layout(location = 1) in vec4 tangent;
layout(location = 2) in vec4 bitangent;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec3 coord;
layout(location = 5) in vec3 world_normal;

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
    float D = clamp(calculate_diffuse(L, world_normal.xyz),0.0,1.0);
    vec3 object = vec3(1,1,1);

    vec3 wireframe = vec3(0,0,0);
    vec3 final_color = ambient*object + D*object;
    float d = min(coord.x, coord.y);
    d = min(d, coord.z);
    d = smoothstep(0.01, 0.1, d);

    color = vec4(mix(final_color, wireframe, 1.0 - d), 1);
}
