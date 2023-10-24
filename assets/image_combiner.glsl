#version 460


layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;


layout(set = 0, binding = 0, rgba8) uniform image2D current_image;

layout(set = 0, binding = 1, rgba8) uniform image2D new_image;


layout(push_constant) uniform PushConstants {
    uint frame;
    uint image_width;
    uint image_height;
}push_constants;




void main() {

    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.image_width * push_constants.image_height) {
        return;
    }

    ivec2 pos = ivec2(id % push_constants.image_width, id / push_constants.image_width);

    vec3 prev_col = imageLoad(current_image, pos).xyz;
    vec3 new_col = imageLoad(new_image, pos).xyz;
    vec4 col = vec4(mix(new_col, prev_col, 1 / (push_constants.frame + 1)), 1);


    imageStore(current_image, pos, col);
}