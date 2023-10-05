use std::time::Instant;
use graphics::*;
use lighting_models::raytracing::*;

const IMAGE_SIZE: [u32; 2] = [1500, 1000];

fn main() {
    let (mut event_loop, vulkano_context, mut vulkano_windows, window_ids, commands_allocator, descriptor_set_allocator) = get_general_graphics_data(vec![("Scene".to_string(), IMAGE_SIZE[0] as f32, IMAGE_SIZE[1] as f32, false)], gen_swapchain_func!(Format::B8G8R8A8_UNORM));
    let mut gui = Vec::new();

    // let mut camera = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), Some(10.0), None);
    let mut camera = Camera::new(Some([-5.0, 10.0, 0.0]), Some([1.0, -1.0, 0.0]), Some(10.0), None);
    camera.controllable();

    let meshes = load_obj("assets/island.obj");
    let mesh_data = vec![
        RayTracingMesh{mesh: meshes[0].clone(), material: RayTraceMaterial{colour: [0.5; 3].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}}
    ];

    let mut last_frame_time = Instant::now();

    let scene_window_id = window_ids[0];
    

    let mut compute_pipeline = RayTracePipeine::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        IMAGE_SIZE
    );
    compute_pipeline.init_data(&vulkano_context, 1.0, 2.0, camera.up, 25, 0.001, 50);
    // compute_pipeline.init_data(&vulkano_context, 1.0, 2.0, camera.up, 500, 0.002, 50);

    let spheres = vec![
        // Sphere {centre: [0, -20, 0].into(), radius: 20.0, material: RayTraceMaterial {colour: [0.7, 0.1, 0.7].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}},
        // Sphere {centre: [2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1, 0, 0].into(), roughness: 0.2, metalic: 1.0, ..Default::default()}},
        // Sphere {centre: [-2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [0.0, 0.5, 0.5].into(), roughness: 0.2, metalic: 0.5, ..Default::default()}},
        // Sphere {centre: [0, 1, 0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1.0, 0.4, 0.4].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}},
        Sphere {centre: [0, 7, 0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1.0; 3].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}}
    ];

    compute_pipeline.update_spheres(&vulkano_context, spheres);
    compute_pipeline.update_meshes(&vulkano_context, &mesh_data);

    let graphics_pipeline = RenderPassOverFrame::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        Format::B8G8R8A8_UNORM
    );

    let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
    let before_compute = renderer.acquire().unwrap();
    let after_compute = compute_pipeline.compute(before_compute, &camera);
    let after_render = graphics_pipeline.render(after_compute, compute_pipeline.image(), renderer.swapchain_image_view());
    renderer.present(after_render, true);
    

    loop {
        if !generic_winit_event_handling_with_camera(&mut event_loop, &mut vulkano_windows, &mut gui, (&mut camera, &scene_window_id)) {break;}

        // let frame_time = last_frame_time.elapsed().as_secs_f32();
        // if frame_time > 1.0 / 60.0 {
        //     last_frame_time = Instant::now();

        //     let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
        //     let before_compute = renderer.acquire().unwrap();
        //     let after_compute = compute_pipeline.compute(before_compute, &camera);
        //     let after_render = graphics_pipeline.render(after_compute, compute_pipeline.image(), renderer.swapchain_image_view());
        //     renderer.present(after_render, true);

        //     camera.do_move(frame_time);
        // }

    }
}