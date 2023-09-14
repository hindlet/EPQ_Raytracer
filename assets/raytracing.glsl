#version 460

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct Ray {
    vec4 origin;
    vec4 dir;
    vec4 img_pos;
};

vec3 ray_at(Ray r, float dist) {
    return vec3(r.origin) + vec3(r.dir) * dist;
}

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
    int num_rays;
    int num_spheres;
} push_constants;

const vec3 SPHERE_COLOUR = vec3(1.0);

// output: (normal, hit_dist)
vec4 intersecting_sphere(Sphere s, Ray r) {
    vec3 l = vec3(r.origin) - s.centre;
    
    float a = dot(vec3(r.dir), vec3(r.dir));
    float half_b = dot(vec3(r.dir), l);
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
    for (int i = 0; i < push_constants.num_spheres; i++) {
        vec4 hit_info = intersecting_sphere(spheres[i], r);
        if (hit_info.w >= 0.0) {
            return 0.5*vec3(hit_info.x+1, hit_info.y+1, hit_info.z+1);
        }
    }
    
    float a = 0.5*(r.dir.y + 1.0);
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