#version 460
#define FLT_MAX 3.402823466e+38
#define M_PI 3.1415926535897932384626433832795
#define UINT_MAX 4294967295

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

/// RANDOM FUNCTIONS

// Hash function www.cs.ubc.ca/~rbridson/docs/schechter-sca08-turbulence.pdf
uint hash(inout uint state) {
    state ^= 2747636419u;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    state ^= state >> 16;
    state *= 2654435769u;
    return state;
}

float scaleToRange01(uint state) {
    return state / 4294967295.0;
}

// Random value in normal distribution (with mean=0 and sd=1)
float RandomValueNormalDistribution(inout uint state) {
    // Thanks to https://stackoverflow.com/a/6178290
    float theta = 2 * 3.1415926 * scaleToRange01(hash(state));
    float rho = sqrt(-2 * log(scaleToRange01(hash(state))));
    return rho * cos(theta);
}

vec3 RandomPointOnUnitSphere(inout uint state) {
    float x = RandomValueNormalDistribution(state);
    float y = RandomValueNormalDistribution(state);
    float z = RandomValueNormalDistribution(state);
    return normalize(vec3(x, y ,z));
}

vec3 RandomPointOnHemisphere(inout uint state, vec3 normal) {
    vec3 dir = RandomPointOnUnitSphere(state);
    if (dot(dir, normal) > 0) {return dir;}
    else {return -dir;}
}


/// STRUCTS

struct RayTracingMaterial {
    vec4 colour;
    vec4 emission; /// vec3 colour, float strength at 1m
    vec4 settings; // roughness, metalic
};

RayTracingMaterial empty_mat() {
    return RayTracingMaterial (
        vec4(0, 0, 0, 1),
        vec4(0),
        vec4(0)
    );
}


struct Ray {
    vec4 sample_centre; // relative to line (1, 0, 0)
    vec4 img_pos;
};


struct Sphere {
    vec3 centre;
    float radius;
    RayTracingMaterial material;
};

struct RayHit {
    vec3 hit_normal;
    vec3 hit_pos;
    float hit_dist;
    RayTracingMaterial hit_mat;
};

RayHit empty_hit() {
    return RayHit (
        vec3(0),
        vec3(0),
        FLT_MAX,
        empty_mat()
    );
}


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
    int max_bounces;
    bool use_environment_light;

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

vec3 colour_to_gamma_two(vec3 colour) {
    return vec3(sqrt(colour.x), sqrt(colour.y), sqrt(colour.z));
}

vec3 lerp(vec3 a, vec3 b, float x) {
    return (a + b) * x;
}


RayHit intersecting_sphere(Sphere s, vec3 root_pos, vec3 dir) {
    vec3 l = root_pos - s.centre;
    
    float a = dot(dir, dir);
    float half_b = dot(dir, l);
    float c = dot(l, l) - s.radius * s.radius;
    float discriminant = half_b * half_b - a * c;

    if (discriminant >= 0) {
        float dist = (-half_b - sqrt(discriminant)) / a;
        vec3 pos = ray_at(root_pos, dir, dist);
        return RayHit(
            pos - s.centre,
            pos,
            dist,
            s.material
        );
    } else {
        return empty_hit();
    }
}


RayHit world_hit(vec3 root_pos, vec3 dir) {
    RayHit closest = empty_hit();

    // check spheres
    for (int i = 0; i < push_constants.num_spheres; i++) {
        RayHit hit_info = intersecting_sphere(spheres[i], root_pos, dir);
        if (hit_info.hit_dist >= 0.0 && hit_info.hit_dist < closest.hit_dist && hit_info.hit_dist > 0.001) {
            closest = hit_info;
        }
    }
   

    return closest;
}

vec3 environment_light(vec3 dir) {
    if (!push_constants.use_environment_light) {return vec3(0);}
    float a = 0.5*(dir.y + 1.0);
    return (1.0-a)*vec3(1.0) + a*vec3(0.5, 0.7, 1.0);
}

vec3 adjust_dir(vec3 dir, vec3 normal, RayTracingMaterial mat, inout uint state) {
    vec3 scatter = normalize(lerp(normal, reflect(dir, normal), mat.settings.y) + RandomPointOnUnitSphere(state) * mat.settings.x);
    return scatter;
}


vec3 trace_ray(vec3 root_pos, vec3 dir, inout uint state) {
    vec3 light = vec3(0);
    vec3 colour = vec3(1);

    vec3 ray_pos = root_pos;
    vec3 ray_dir = dir;

    for (int i = 0; i <= push_constants.max_bounces; i++) {
        RayHit hit = world_hit(ray_pos, ray_dir);
        if (hit.hit_dist < FLT_MAX) {
            ray_pos = hit.hit_pos;
            ray_dir = adjust_dir(ray_dir, hit.hit_normal, hit.hit_mat, state);
            
            colour *= vec3(hit.hit_mat.colour);
            // colour *= 0.5;
        }
        else {
            light += environment_light(ray_dir) * colour_to_gamma_two(colour);
            break;
        }
    }

    // light = environment_light(ray_dir);

    return light;
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

        colour += trace_ray(vec3(push_constants.cam_pos), normalize(dir), state);
    }

    colour /= push_constants.num_samples;
    imageStore(img, ivec2(rays[id].img_pos.xy), vec4(colour, 1.0));
}