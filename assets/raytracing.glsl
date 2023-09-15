#version 460
#define FLT_MAX 3.402823466e+38

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;


/// PRNG

#define hashi(x) lowbias32(x)

#define hash(x) (float( hashi(x) ) / float( 0xffffffffU ))

// https://www.shadertoy.com/view/WttXWX
// bias: 0.17353355999581582 ( very probably the best of its kind )
uint lowbias32(uint x)
{
    x ^= x >> 16;
    x *= 0x7feb352dU;
    x ^= x >> 15;
    x *= 0x846ca68bU;
    x ^= x >> 16;
    return x;
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
    vec4 pixel_dims;

    int num_rays;
    int num_spheres;
    int num_samples;

} push_constants;


/// FUNCTIONS

vec3 ray_at(vec3 root_pos, vec3 dir, float dist) {
    return root_pos + dir * dist;
}

vec3 get_ray_dir(mat3 allignment_mat, vec3 ray_centre, ivec2 img_pos, uint ray_id, uint sample_id, float pixel_x, float pixel_y) {
    uint number = ray_id + sample_id;
    float x_offset = hash(number + hashi(uint(img_pos.x))) * 2 - 1;
    float y_offset = hash(number + hashi(uint(img_pos.y))) * 2 - 1;

    vec3 ray_pos = ray_centre + vec3(0, 0, 0.5) * pixel_x * x_offset + vec3(0, 0.5, 0) * (pixel_y) * y_offset;
    return normalize(allignment_mat * ray_centre);
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
    for (int i = 0; i < push_constants.num_samples; i++) {
        vec3 dir = get_ray_dir(mat3(push_constants.cam_alignment_mat), vec3(rays[id].sample_centre), ivec2(rays[id].img_pos.xy), id, i, push_constants.pixel_dims.x, push_constants.pixel_dims.y);
        colour += ray_colour(vec3(push_constants.cam_pos), dir);
    }

    
    colour /= push_constants.num_samples;
    imageStore(img, ivec2(rays[id].img_pos.xy), vec4(colour, 1.0));
}