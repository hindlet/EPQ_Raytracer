#version 460

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct Ray {
    vec4 dir;
    vec4 pos;
};

layout(set = 0, binding = 0, rgba8) uniform image2D img;


layout(set = 0, binding = 1) buffer Rays {
    Ray[] rays;
};

layout(push_constant) uniform PushConstants {
    int num_rays;
} push_constants;


vec3 ray_colour(Ray r) {
    float a = 0.5*(r.dir.y + 1.0);
    return (1.0-a)*vec3(1.0, 1.0, 1.0) + a*vec3(0.5, 0.7, 1.0);
    // return vec3(0.0, 1.0, 0.0);
}


void main() {
    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.num_rays) {
        return;
    }


    vec3 colour = ray_colour(rays[id]);
    imageStore(img, ivec2(rays[id].pos.xy), vec4(colour, 1.0));
}