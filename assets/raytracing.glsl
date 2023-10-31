#version 460
#define FLT_MAX 3.402823466e+38
#define FLT_MIN 1.175494e-38
#define M_PI 3.1415926535897932384626433832795
#define UINT_MAX 4294967295

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

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
    vec4 emission; /// vec3 colour, float strength
    vec4 settings; // roughness, metalic, fuzz
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
};


struct Sphere {
    vec3 centre;
    float radius;
    RayTracingMaterial material;
};


// made of three points: a, b, c
struct Triangle {
    vec4 a; // first point
    vec4 edge_one; // edge one (b - a)
    vec4 edge_two; // edge two (c - a)
    vec4 normal; // normal (edge_one x edge_two)
};

struct Mesh {
    vec3 min_point;
    uint first_index;
    vec3 max_point;
    uint len;
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

layout(set = 0, binding = 3) buffer Triangles {
    Triangle[] triangles;
};

layout(set = 0, binding = 4) buffer Meshes {
    Mesh[] meshes;
};

layout(push_constant) uniform PushConstants {
    vec4 cam_pos;
    mat4 cam_alignment_mat;

    int num_rays;
    int num_spheres;
    int num_meshes;

    int num_samples;
    float jitter_size;
    int max_bounces;
    bool use_environment_light;
    uint rng_offset;

    bool init;
    uint width;
    uint height;

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

bool intersecting_aabb(vec3 min_point, vec3 max_point, vec3 root_pos, vec3 dir) {
    vec3 inv_dir = vec3(1) / dir;
    float d_max;
    float d_min;

    d_max = (((inv_dir.x < 0) ? min_point.x : max_point.x) - root_pos.x) * inv_dir.x;
    d_min = (((inv_dir.x < 0) ? max_point.x : min_point.x) - root_pos.x) * inv_dir.x;
    if (d_max > 0 || d_min > 0) {return true;}

    d_max = max(d_max, (((inv_dir.y < 0) ? min_point.y : max_point.y) - root_pos.y) * inv_dir.y );
    d_min = min(d_max, (((inv_dir.y < 0) ? max_point.y : min_point.y) - root_pos.y) * inv_dir.y );
    if (d_max > 0 || d_min > 0) {return true;}

    d_max = max(d_max, (((inv_dir.z < 0) ? min_point.z : max_point.z) - root_pos.z) * inv_dir.z );
    d_min = max(d_min, (((inv_dir.z < 0) ? max_point.z : min_point.z) - root_pos.z) * inv_dir.z );
    if (d_max > 0 || d_min > 0) {return true;}

    return false;
}

// (hit_normal, hit dist)
vec4 intersecting_tri(Triangle t, vec3 root_pos, vec3 dir) {

    vec3 normal = vec3(t.normal);

    vec3 ao = root_pos - vec3(t.a);
    vec3 dao = cross(ao, dir);

    float det = -dot(dir, normal);
    if (det == 0) {return vec4(FLT_MAX);}

    float inv_det = 1 / det;
    float dist = dot(ao, normal) * inv_det;
    if (dist < 0) {return vec4(FLT_MAX);}

    float u = dot(vec3(t.edge_two), dao) * inv_det;
    if (u < 0) {return vec4(FLT_MAX);}

    float v = -dot(vec3(t.edge_one), dao) * inv_det;
    if (v < 0) {return vec4(FLT_MAX);}

    float w = 1 - u - v;
    if (w < 0) {return vec4(FLT_MAX);}

    return vec4(normal, dist);
}

RayHit intersecting_mesh(Mesh m, vec3 root_pos, vec3 dir) {

    if (!intersecting_aabb(m.min_point, m.max_point, root_pos, dir)) {return empty_hit();}

    vec4 closest = vec4(FLT_MAX);

    for (uint i = 0; i < m.len; i++) {
        vec4 hit_info = intersecting_tri(triangles[i + m.first_index], root_pos, dir);
        if (hit_info.w > 0.001 && hit_info.w < closest.w) {
            closest = hit_info;
        }
    }

    if (closest.w == FLT_MAX) {return empty_hit();}

    return RayHit(
        vec3(closest),
        ray_at(root_pos, dir, closest.w),
        closest.w,
        m.material
    );
}


RayHit world_hit(vec3 root_pos, vec3 dir) {
    RayHit closest = empty_hit();

    // check spheres
    for (int i = 0; i < push_constants.num_spheres; i++) {
        RayHit hit_info = intersecting_sphere(spheres[i], root_pos, dir);
        if (hit_info.hit_dist > 0.001 && hit_info.hit_dist < closest.hit_dist) {
            closest = hit_info;
        }
    }

    // check meshes
    for (int i = 0; i < push_constants.num_meshes; i++) {
        RayHit hit_info = intersecting_mesh(meshes[i], root_pos, dir);
        if (hit_info.hit_dist > 0.001 && hit_info.hit_dist < closest.hit_dist) {
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


/// there is a problem here, idk why
vec3 adjust_dir(vec3 dir, vec3 normal, RayTracingMaterial mat, inout uint state) {

    vec3 diffuse_dir = normalize(normal + RandomPointOnUnitSphere(state) * mat.settings.x); // lambertian
    vec3 specular_dir = reflect(dir, normal); // metal
    vec3 fuzz = RandomPointOnUnitSphere(state) * mat.settings.z; // metal fuzz
    
    vec3 new_dir = normalize(mix(diffuse_dir, specular_dir, mat.settings.y) + fuzz);
    return new_dir;
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
            
            vec3 emitted_light = vec3(hit.hit_mat.emission) * hit.hit_mat.emission.w;
            light += emitted_light * colour;
            colour *= vec3(hit.hit_mat.colour);

            // Random early exit if ray colour is nearly 0 (can't contribute much to final result)
            float p = max(colour.x, max(colour.y, colour.z));
            if (scaleToRange01(hash(state)) >= p) {
                break;
            }
            // colour *= 1.0f / p; 
        }
        else {
            light += environment_light(ray_dir);
            break;
        }
    }

    // light = environment_light(ray_dir);

    return light * colour_to_gamma_two(colour);
}


void main() {
    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;

    if (x > push_constants.width || y > push_constants.height) {
        return;
    }

    if (push_constants.init) {
        imageStore(img, ivec2(x, y), vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }

    // if (push_constants.rng_offset % 2 == 0) {
    //     imageStore(img, ivec2(x, y), vec4(0.0, 0.0, 1.0, 1.0));
    //     return;
    // } else {
    //     imageStore(img, ivec2(x, y), vec4(1.0, 0.0, 1.0, 1.0));
    //     return;
    // }

    uint id = x + y * push_constants.width;

    vec3 colour = vec3(0);
    uint state = push_constants.rng_offset * 719393 + id;
    for (int i = 0; i < push_constants.num_samples; i++) {
        
        vec3 dir = get_ray_dir(vec3(rays[id].sample_centre), state);

        colour += trace_ray(vec3(push_constants.cam_pos), normalize(dir), state);
    }

    colour /= push_constants.num_samples;
    imageStore(img, ivec2(x, y), vec4(colour, 1.0));
}