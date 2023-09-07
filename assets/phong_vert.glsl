#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 colour;

layout(location = 0) out vec3 v_colour;
layout(location = 1) out vec3 f_pos;
layout(location = 2) out vec3 f_normal;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    gl_Position = uniforms.proj * uniforms.view * uniforms.world * vec4(position, 1.0);
    v_colour = vec3(colour);
    f_normal = mat3(transpose(inverse(uniforms.world))) * normal;
    f_pos = vec3(uniforms.world * vec4(position, 1.0));
}