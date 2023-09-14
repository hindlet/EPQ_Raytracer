#version 460
#define FLT_MAX 3.402823466e+38

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;



struct Ray {
    vec4 dir; // relative to line (1, 0, 0)
    vec4 img_pos;
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
    vec3 cam_pos;
    int num_rays;
    int num_spheres;
    mat3 cam_alignment_mat;
} push_constants;


vec3 ray_at(Ray r, float dist) {
    return push_constants.cam_pos + (vec3(r.dir) * push_constants.cam_alignment_mat) * dist;
}


// output: (normal, hit_dist)
vec4 intersecting_sphere(Sphere s, Ray r) {
    vec3 l = push_constants.cam_pos - s.centre;

    vec3 dir = vec3(r.dir) * push_constants.cam_alignment_mat;
    
    float a = dot(dir, dir);
    float half_b = dot(dir, l);
    float c = dot(l, l) - s.radius * s.radius;
    float discriminant = half_b * half_b - a * c;

    if (discriminant >= 0) {
        float dist = (-half_b - sqrt(discriminant)) / a;
        return vec4(
            ray_at(r, dist) - s.centre,
            dist
        );
    } else {
        return vec4(-1);
    }
}


vec3 ray_colour(Ray r) {
    
    // sphere intersections
    vec4 closest = vec4(0, 0, 0, FLT_MAX);
    for (int i = 0; i < push_constants.num_spheres; i++) {
        vec4 hit_info = intersecting_sphere(spheres[i], r);
        if (hit_info.w >= 0.0 && hit_info.w < closest.w) {
            closest = hit_info;
        }
    }
   if (closest.w != FLT_MAX) {
        return 0.5*vec3(closest.x+1, closest.y+1, closest.z+1);
    }
    
    float a = 0.5*((vec3(r.dir) * push_constants.cam_alignment_mat).y + 1.0);
    return (1.0-a)*vec3(0.0) + a*vec3(0.5, 0.7, 1.0);
}


void main() {
    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.num_rays) {
        return;
    }


    vec3 colour = ray_colour(rays[id]);
    imageStore(img, ivec2(rays[id].img_pos.xy), vec4(colour, 1.0));
}