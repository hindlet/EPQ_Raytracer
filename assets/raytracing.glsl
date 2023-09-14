#version 460

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct Ray {
    vec4 dir;
    vec4 pos;
};

struct Sphere {
    vec3 centre;
    float radius;
};

layout(set = 0, binding = 0, rgba8) uniform image2D img;


layout(set = 0, binding = 1) buffer Rays {
    Ray[] rays;
};

layout(set = 0, binding = 2) buffer Spheres {
    Sphere[] spheres;
};

layout(push_constant) uniform PushConstants {
    vec3 camera_pos;

    int num_rays;
    int num_spheres;
} push_constants;

const vec3 SPHERE_COLOUR = vec3(1.0);

bool intersecting_sphere(Sphere s, Ray r, vec3 root_pos) {
    vec3 l = root_pos - s.centre;
    
    float a = dot(vec3(r.dir.xyz), vec3(r.dir.xyz));
    float b = 2 * dot(vec3(r.dir.xyz), l);
    float c = dot(l, l) - s.radius * s.radius;
    return b * b - 4 * a * c >= 0;
}


vec3 ray_colour(Ray r) {
    float a = 0.5*(r.dir.y + 1.0);
    
    for (int i = 0; i < push_constants.num_spheres; i++) {
        if (intersecting_sphere(spheres[i], r, push_constants.camera_pos)) {
            return vec3(1);
        }
    }
    
    return (1.0-a)*vec3(0.0) + a*vec3(0.5, 0.7, 1.0);
}


void main() {
    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.num_rays) {
        return;
    }


    vec3 colour = ray_colour(rays[id]);
    imageStore(img, ivec2(rays[id].pos.xy), vec4(colour, 1.0));
}