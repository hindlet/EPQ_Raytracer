#version 460
#define FLT_MAX 3.402823466e+38
#define M_PI 3.1415926535897932384626433832795
#define UINT_MAX 4294967295

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;



// Hash function www.cs.ubc.ca/~rbridson/docs/schechter-sca08-turbulence.pdf
uint hash(inout uint state)
{
    state ^= 2747636419u;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    return state;
}

float scaleToRange01(uint state)
{
    return state / 4294967295.0;
}


/// STRUCTS


struct Ray {
    vec4 sample_centre; // relative to line (1, 0, 0)
    vec4 img_pos;
};


struct Sphere {
    vec3 centre;
    float radius;
};


/// BUFFERS

layout(set = 0, binding = 0, rgba8) uniform image2D img;


layout(set = 0, binding = 1) buffer Rays {
    Ray[] rays;
};

layout(set = 0, binding = 2) buffer Spheres {
    Sphere[] spheres;
};

layout(push_constant) uniform PushConstants {
    vec4 cam_pos;
    mat4 cam_alignment_mat;

    int num_rays;
    int num_spheres;
    int num_samples;
    float jitter_size;

} push_constants;


/// FUNCTIONS

vec3 ray_at(vec3 root_pos, vec3 dir, float dist) {
    return root_pos + dir * dist;
}

vec3 get_ray_dir(vec3 sample_centre, inout uint state) {
    float random = scaleToRange01(hash(state)) * 2 * M_PI;
    vec3 new_centre = sample_centre + cos(random) * vec3(0, 0, 1) * push_constants.jitter_size * sqrt(scaleToRange01(hash(state))) + sin(random) * vec3(0, 1, 0) * push_constants.jitter_size * sqrt(scaleToRange01(hash(state)));
    return normalize(mat3(push_constants.cam_alignment_mat) * new_centre);
}


// output: (normal, hit_dist)
vec4 intersecting_sphere(Sphere s, vec3 root_pos, vec3 dir) {
    vec3 l = root_pos - s.centre;
    
    float a = dot(dir, dir);
    float half_b = dot(dir, l);
    float c = dot(l, l) - s.radius * s.radius;
    float discriminant = half_b * half_b - a * c;

    if (discriminant >= 0) {
        float dist = (-half_b - sqrt(discriminant)) / a;
        return vec4(
            ray_at(root_pos, dir, dist) - s.centre,
            dist
        );
    } else {
        return vec4(-1);
    }
}


vec3 ray_colour(vec3 root_pos, vec3 dir) {
    
    // sphere intersections
    vec4 closest = vec4(0, 0, 0, FLT_MAX);
    for (int i = 0; i < push_constants.num_spheres; i++) {
        vec4 hit_info = intersecting_sphere(spheres[i], root_pos, dir);
        if (hit_info.w >= 0.0 && hit_info.w < closest.w) {
            closest = hit_info;
        }
    }
   if (closest.w != FLT_MAX) {
        return 0.5*vec3(closest.x+1, closest.y+1, closest.z+1);
    }
    
    float a = 0.5*(dir.y + 1.0);
    return (1.0-a)*vec3(0.0) + a*vec3(0.5, 0.7, 1.0);
}


void main() {
    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.num_rays) {
        return;
    }

    vec3 colour = vec3(0);
    uint state = id;
    for (int i = 0; i < push_constants.num_samples; i++) {
        
        vec3 dir = get_ray_dir(vec3(rays[id].sample_centre), state);

        colour += ray_colour(vec3(push_constants.cam_pos), normalize(dir));
    }

    colour /= push_constants.num_samples;
    imageStore(img, ivec2(rays[id].img_pos.xy), vec4(colour, 1.0));
}