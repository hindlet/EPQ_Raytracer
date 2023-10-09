#version 460


layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;


layout(set = 0, binding = 0, rgba8) uniform image2D current_image;

layout(set = 0, binding = 1, rgba8) uniform image2D new_image;


layout(push_constant) uniform PushConstants {
    uint num_images;
    uint image_width;
    uint image_height;
}push_constants;



void main() {

    uint id = gl_GlobalInvocationID.x;

    if (id >= push_constants.image_width * push_constants.image_height) {
        return;
    }

    ivec2 pos = ivec2(id % push_constants.image_width, id / push_constants.image_width);
    vec4 summed_colour = imageLoad(current_image, pos) * push_constants.num_images;
    summed_colour += imageLoad(new_image, pos);


    imageStore(current_image, pos, summed_colour / (push_constants.num_images + 1));
}