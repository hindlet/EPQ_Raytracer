use std::time::Instant;
use graphics::*;
use maths::Vector3;
use lighting_models::{gen_icosphere, blinn_phong::*};

const AMBIENT_STRENGTH: f32 = 0.1 * 1.0;
const DIFFUSE_STRENGTH: f32 = 0.5 * 1.0;
const SPECULAR_STRENGTH: f32 = 2.0 * 1.0;


fn main() {

    let (mut event_loop, vulkano_context, mut vulkano_windows, window_ids, commands_allocator, descriptor_set_allocator) = get_general_graphics_data(vec![("Scene".to_string(), 1500.0, 1000.0, false)], gen_swapchain_func!(Format::B8G8R8A8_SRGB));
    let uniform_allocator = create_uniform_buffer_allocator(vulkano_context.memory_allocator());

    let large_sphere = gen_icosphere(20.0, Vector3::Y * -20.0, [1.0, 1.0, 1.0, 1.0], 5);
    let red_sphere = gen_icosphere(1.0, Vector3::X * -2.5 + Vector3::Y * 0.75, [1.0, 0.1, 0.3, 1.0], 3);
    let blue_sphere = gen_icosphere(1.0, Vector3::X * 2.5 + Vector3::Y * 0.75, [0.5, 0.7, 1.0, 1.0], 3);
    let green_sphere = gen_icosphere(1.0, Vector3::Y, [0.1, 1.0, 0.4, 1.0], 3);

    let mut total_mesh = large_sphere;
    total_mesh.add(red_sphere);
    total_mesh.add(blue_sphere);
    total_mesh.add(green_sphere);
    let (total_vertices, total_normals, total_indices) = total_mesh.components();

    let vertex_buffer = create_shader_data_buffer(total_vertices, &vulkano_context, BufferType::Vertex);
    let normal_buffer = create_shader_data_buffer(total_normals, &vulkano_context, BufferType::Normal);
    let index_buffer = create_shader_data_buffer(total_indices, &vulkano_context, BufferType::Index);

    let lights = vec![
        Light{pos: [3, 2, 0].into(), strength: 0.3, colour: [1, 1, 1].into()}, // white
        Light{pos: [0, 3, 0].into(), strength: 0.5, colour: [1, 0, 1].into()}, // purple
        Light{pos: [-3, 5, 0].into(), strength: 0.8, colour: [0, 1, 1].into()}, // whatever blue + green is
        Light{pos: [3.75, 0.0, 0.0].into(), strength: 0.7, colour: [0, 1, 0].into()}, // green
    ];

    let light_buffer = get_light_buffer(&uniform_allocator, lights);


    let mut gui = Vec::new();

    let mut camera = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), Some(10.0), None);

    let mut last_frame_time = Instant::now();

    let scene_window_id = window_ids[0];


    let mut pipeline = BlinnPhongPipeline::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        Some(SampleCount::Sample4),
    );

    let uniforms = get_uniforms(vulkano_windows.get_renderer_mut(scene_window_id).unwrap().swapchain_image_size(), &uniform_allocator, &camera, AMBIENT_STRENGTH, DIFFUSE_STRENGTH, SPECULAR_STRENGTH);

    loop {
        if !generic_winit_event_handling_with_camera(&mut event_loop, &mut vulkano_windows, &mut gui, (&mut camera, &scene_window_id)) {break;}

        let frame_time = last_frame_time.elapsed().as_secs_f32();
        if frame_time > 1.0 / 60.0 {
            last_frame_time = Instant::now();

            let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
            let before_future = renderer.acquire().unwrap();
            let after_future = pipeline.draw(before_future, renderer.swapchain_image_view(), &vertex_buffer, &normal_buffer, &index_buffer, &uniforms, &light_buffer);
            renderer.present(after_future, true);

            camera.do_move(frame_time);
        }

    }

}

