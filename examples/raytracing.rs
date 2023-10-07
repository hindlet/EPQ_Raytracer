use std::time::Instant;
use graphics::*;
use lighting_models::raytracing::*;

const IMAGE_SIZE: [u32; 2] = [1500, 1000];

fn main() {
    let (mut event_loop, vulkano_context, mut vulkano_windows, window_ids, commands_allocator, descriptor_set_allocator) = get_general_graphics_data(vec![("Scene".to_string(), IMAGE_SIZE[0] as f32, IMAGE_SIZE[1] as f32, false)], gen_swapchain_func!(Format::B8G8R8A8_UNORM));
    let mut gui = Vec::new();

    // let mut camera = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), Some(10.0), None);
    let mut camera = Camera::new(Some([-4.4, 1.6, -11.9]), Some([0.5, 0.0, 1.3]), Some(10.0), None);
    let mut camera = Camera::new(Some([30.9, -6.5, 11.4]), Some([-1.25, 0.0, -0.6]), Some(10.0), None);
    // camera.controllable();

    // let meshes = load_obj("assets/island.obj");
    // let mesh_data = vec![
    //     RayTracingMesh{mesh: meshes[0].clone(), material: RayTraceMaterial{colour: [0.5; 3].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}}, // island
    //     RayTracingMesh{mesh: meshes[1].clone(), material: RayTraceMaterial{emmision: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 15.0].into(), colour: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0].into(), ..Default::default()}}, // water
    //     RayTracingMesh{mesh: meshes[2].clone(), material: RayTraceMaterial{colour: [38.0 / 255.0, 21.0 / 255.0, 5.0 / 255.0].into(), roughness: 0.5, metalic: 1.0, ..Default::default()}}, // tree
    //     RayTracingMesh{mesh: meshes[3].clone(), material: RayTraceMaterial{colour: [0.5, 1.0, 0.0].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}} // leaves
    // ];
    let meshes = load_obj("assets/Cave.obj");
    let mesh_data = vec![
        RayTracingMesh{mesh: meshes[4].clone(), material: RayTraceMaterial{colour: [0.5; 3].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}}, // island
        RayTracingMesh{mesh: meshes[5].clone(), material: RayTraceMaterial{emmision: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 5.0].into(), colour: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0].into(), ..Default::default()}}, // water
        RayTracingMesh{mesh: meshes[0].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        // RayTracingMesh{mesh: meshes[1].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        // RayTracingMesh{mesh: meshes[2].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        // RayTracingMesh{mesh: meshes[3].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        // RayTracingMesh{mesh: meshes[2].clone(), material: RayTraceMaterial{colour: [207.0 / 255.0, 190.0 / 255.0, 145.0 / 255.0].into(), roughness: 0.5, metalic: 0.0, ..Default::default()}}, // mushroom
        // RayTracingMesh{mesh: meshes[3].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}}, // dots
        // RayTracingMesh{mesh: meshes[4].clone(), material: RayTraceMaterial{colour: [207.0 / 255.0, 190.0 / 255.0, 145.0 / 255.0].into(), roughness: 0.5, metalic: 0.0, ..Default::default()}}, // mushroom 2
        // RayTracingMesh{mesh: meshes[5].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}} // dots 2
    ];


    let mut last_frame_time = Instant::now();

    let scene_window_id = window_ids[0];
    

    let mut compute_pipeline = RayTracePipeine::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        IMAGE_SIZE
    );
    // compute_pipeline.init_data(&vulkano_context, 1.0, 2.0, camera.up, 1, 0.0, 2, true);
    compute_pipeline.init_data(&vulkano_context, 1.0, 2.0, camera.up, 10, 0.001, 50, false);
    // compute_pipeline.init_data(&vulkano_context, 1.0, 2.0, camera.up, 500, 0.002, 50, true);
    // compute_pipeline.init_data(&vulkano_context, 1.0, 2.0, camera.up, 25, 0.002, 50, false);

    let spheres = vec![
        Sphere {centre: [0, -20, 0].into(), radius: 20.0, material: RayTraceMaterial {colour: [0.7, 0.1, 0.7].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}},
        Sphere {centre: [2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1, 0, 0].into(), roughness: 0.2, metalic: 1.0, ..Default::default()}},
        Sphere {centre: [-2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [0.0, 0.5, 0.5].into(), roughness: 0.2, metalic: 0.5, ..Default::default()}},
        Sphere {centre: [0, 1, 0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1.0, 0.4, 0.4].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}},
    ];

    // compute_pipeline.update_spheres(&vulkano_context, spheres);
    compute_pipeline.update_meshes(&vulkano_context, &mesh_data);

    let graphics_pipeline = RenderPassOverFrame::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        Format::B8G8R8A8_UNORM
    );

    let start_time = Instant::now();
    let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
    let before_compute = renderer.acquire().unwrap();
    let after_compute = compute_pipeline.compute(before_compute, &camera);
    let after_render = graphics_pipeline.render(after_compute, compute_pipeline.image(), renderer.swapchain_image_view());
    renderer.present(after_render, true);
    println!("It took {} seconds to render that scene", start_time.elapsed().as_secs_f32());
    

    loop {
        if !generic_winit_event_handling_with_camera(&mut event_loop, &mut vulkano_windows, &mut gui, (&mut camera, &scene_window_id)) {break;}

        let frame_time = last_frame_time.elapsed().as_secs_f32();
        // if frame_time > 1.0 / 60.0 {
        //     last_frame_time = Instant::now();

        //     let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
        //     let before_compute = renderer.acquire().unwrap();
        //     let after_compute = compute_pipeline.compute(before_compute, &camera);
        //     let after_render = graphics_pipeline.render(after_compute, compute_pipeline.image(), renderer.swapchain_image_view());
        //     renderer.present(after_render, true);

        //     camera.do_move(frame_time);
        //     println!("{:?}, {:?}", camera.position, camera.direction);
        // }

    }
}