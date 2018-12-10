#version 450 core
layout(triangles) in;

layout(triangle_strip, max_vertices=3) out;

layout(location = 0 ) in vec4 normal[];
layout(location = 1 ) in vec4 tangent[];
layout(location = 2 ) in vec4 bitangent[];
layout(location = 3 ) in vec2 uv[];
layout(location = 4 ) in vec3 world_normal[];


layout(location = 0) out vec4 out_normal;
layout(location = 1) out vec4 out_tangent;
layout(location = 2) out vec4 out_bitangent;
layout(location = 3) out vec2 out_uv;
layout(location = 4) out vec3 out_world_normal;
layout(location = 5) out vec3 out_coord;

uniform int u_correction = 0;

// Calculates the height between p0 and  the edge formed by p1 and p2
float calculate_height(vec4 p0, vec4 p1, vec4 p2){
    vec4 base = p1 - p2;
    float d = dot(p0 - p2, base);
    vec4 mid = p2 + d * normalize(base);
    return length(mid - p0);
}

void main() {
    vec4 p0 = gl_in[0].gl_Position;
    vec4 p1 = gl_in[1].gl_Position;
    vec4 p2 = gl_in[2].gl_Position;

    gl_Position = p0;
    out_normal = normal[0];
    out_tangent = tangent[0];
    out_bitangent = bitangent[0];
    out_uv = uv[0];
    out_coord = vec3(1,0,0);
    out_world_normal = world_normal[0];

    if (u_correction == 1)
    {
        out_coord = vec3(calculate_height(p0,p1,p2),0,0);
    }
    EmitVertex(); 

    gl_Position = p1;
    out_normal = normal[1];
    out_tangent = tangent[1];
    out_bitangent = bitangent[1];
    out_uv = uv[1];
    out_coord = vec3(0,1,0);
    out_world_normal = world_normal[1];

    // Calculate height
    if (u_correction == 1)
    {
        out_coord = vec3(0,calculate_height(p1,p2,p0),0);
    }
    EmitVertex(); 

    gl_Position = p2;
    out_normal = normal[2];
    out_tangent = tangent[2];
    out_bitangent = bitangent[2];
    out_uv = uv[2];
    out_coord = vec3(0,0,1);
    if(u_correction == 1)
    {
        out_coord = vec3(0,0,calculate_height(p2,p0,p1));
    }
    out_world_normal = world_normal[2];
    EmitVertex(); 

    EndPrimitive();
}