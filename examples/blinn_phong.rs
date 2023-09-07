use std::time::Instant;
use graphics::*;
use maths::Vector3;
use lighting_models::{gen_icosphere, blinn_phong::*};




fn main() {

    let (mut event_loop, vulkano_context, mut vulkano_windows, window_ids, commands_allocator, descriptor_set_allocator) = get_general_graphics_data(vec![("Scene".to_string(), 1500.0, 1000.0, false)]);
    let uniform_allocator = create_uniform_buffer_allocator(vulkano_context.memory_allocator());


    let sphere = gen_icosphere(1.0, Vector3::X * 3.0, [1.0, 1.0, 1.0, 1.0], 3);
    let mut cube = Mesh::new(test_cube::COLOURED_VERTICES.to_vec(), test_cube::INDICES.to_vec());
    cube.set_normals(test_cube::NORMALS.to_vec());
    
    let mut total_mesh = sphere;
    total_mesh.add(cube);
    let (total_vertices, total_normals, total_indices) = total_mesh.components();

    let vertex_buffer = create_shader_data_buffer(total_vertices, &vulkano_context, BufferType::Vertex);
    let normal_buffer = create_shader_data_buffer(total_normals, &vulkano_context, BufferType::Normal);
    let index_buffer = create_shader_data_buffer(total_indices, &vulkano_context, BufferType::Index);

    let light_buffer = get_light_buffer(&uniform_allocator, vec![Light{pos: [3, 2, 0].into(), strength: 0.3, colour: [1, 1, 1].into()}, Light{pos: [0, -2, 0].into(), strength: 0.7, colour: [0, 0, 1].into()}]);


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

    loop {
        if !generic_winit_event_handling_with_camera(&mut event_loop, &mut vulkano_windows, &mut gui, (&mut camera, &scene_window_id)) {break;}

        let frame_time = last_frame_time.elapsed().as_secs_f32();
        if frame_time > 1.0 / 60.0 {
            last_frame_time = Instant::now();

            let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
            let before_future = renderer.acquire().unwrap();
            let after_future = pipeline.draw(before_future, renderer.swapchain_image_view(), &vertex_buffer, &normal_buffer, &index_buffer, &get_uniforms(renderer.swapchain_image_size(), &uniform_allocator, &camera), &light_buffer);
            renderer.present(after_future, true);

            camera.do_move(frame_time);
        }

    }

}

