#version 450 core

layout(location = 0) in vec4 normal;
layout(location = 1) in vec4 tangent;
layout(location = 2) in vec4 bitangent;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec3 world_normal;
layout(location = 5) in vec3 coord;

out vec4 color;

uniform float u_thickness = 0.005;
uniform float u_falloff = 0.005;

uniform vec3 u_object_color = vec3(1,1,1);
uniform vec3 u_wireframe_color = vec3(0,0,0);

uniform vec3 light = vec3(0.33, 0.33, 0.33);
uniform vec3 light_color = vec3(1,1,1);
uniform float ambient_strength = 0.05;
float calculate_diffuse(vec3 L, vec3 N){
    return dot(N,L);
}

void main() {
    vec3 ambient = ambient_strength * light_color;

    vec3  L = normalize(light);
    float D = clamp(calculate_diffuse(L, world_normal.xyz),0.0,1.0);

    // Calculate the final "lit" color
    vec3 object = u_object_color;
    vec3 final_color = ambient*object + D*object;

    // Get the minimum distance
    float d = min(coord.x, min(coord.y, coord.z));
    d = smoothstep(u_thickness, u_thickness + u_falloff, d);

    color = vec4(mix(final_color, u_wireframe_color, 1.0 - d), 1);
}
