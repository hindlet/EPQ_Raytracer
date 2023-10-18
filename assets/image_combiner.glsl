#version 460


layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;


layout(set = 0, binding = 0, rgba8) uniform image2D current_image;

layout(set = 0, binding = 1, rgba8) uniform image2D new_image;


layout(push_constant) uniform PushConstants {
    uint frame;
    uint image_width;
    uint image_height;
}push_constants;

vec4 saturate(vec4 initial) {
    return vec4(
        clamp(initial.x, 0, 1),
        clamp(initial.y, 0, 1),
        clamp(initial.z, 0, 1),
        clamp(initial.w, 0, 1)
    );
}



void main() {

    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.image_width * push_constants.image_height) {
        return;
    }

    float weight = 1 / (push_constants.frame + 1);
    ivec2 pos = ivec2(id % push_constants.image_width, id / push_constants.image_width);

    vec4 col = imageLoad(current_image, pos) * weight;
    col += imageLoad(new_image, pos) * (1 - weight);


    imageStore(current_image, pos, saturate(col));
}