#version 460


layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;


layout(set = 0, binding = 0, rgba8) uniform image2D current_image;

layout(set = 0, binding = 1, rgba8) uniform image2D new_image;



layout(push_constant) uniform PushConstants {
    uint frame;
    uint image_width;
    uint image_height;
}push_constants;




void main() {

    uint x = gl_GlobalInvocationID.x;
    uint y = gl_GlobalInvocationID.y;

    if (x > push_constants.image_width || y > push_constants.image_height) {
        return;
    }

    ivec2 pos = ivec2(x, y);

    if (push_constants.frame == 0) {
        imageStore(current_image, pos, vec4(0, 0, 0, 1));
        return;
    }

    vec3 prev_col = imageLoad(current_image, pos).xyz;
    vec3 new_col = imageLoad(new_image, pos).xyz;
    vec3 col = (new_col + prev_col * push_constants.frame) / (push_constants.frame + 1);


    imageStore(current_image, pos, vec4(col, 1));
}