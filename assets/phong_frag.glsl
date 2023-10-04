#version 460

struct Light {
    vec3 position;
    float intensity;
    vec3 colour;
};

layout(location = 0) in vec3 v_colour;
layout(location = 1) in vec3 f_pos;
layout(location = 2) in vec3 f_normal;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform Data {
    vec3 viewpos;
    float ambient_strength;
    float diffuse_strength;
    float specular_strength;
} uniforms;

#define NR_LIGHTS 4 
layout(set = 0, binding = 2) uniform LightData {
    Light lights[NR_LIGHTS];
} light_uniforms;

vec3 calculate_lighting(Light light, vec3 pos, vec3 norm, vec3 view_pos) {
    vec3 light_dir = normalize(light.position - pos);
    float diff = max(dot(norm, light_dir), 0.0);
    vec3 diffuse = diff * light.intensity * light.colour * uniforms.diffuse_strength;

    vec3 view_dir = normalize(view_pos - pos);
    vec3 reflect_dir = reflect(-light_dir, norm);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
    vec3 specular = light.intensity * uniforms.specular_strength * spec * light.colour;

    return diffuse + specular;
}

void main() {
    vec3 ambient = vec3(uniforms.ambient_strength);

    vec3 light = vec3(0);
    vec3 norm = normalize(f_normal);

    for (int i = 0; i < NR_LIGHTS; i++) {
        light += calculate_lighting(light_uniforms.lights[i], f_pos, norm, uniforms.viewpos);
        
    }

    f_color = vec4((ambient + light) * v_colour, 1.0);
}