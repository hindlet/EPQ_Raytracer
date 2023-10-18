use std::time::Instant;
use graphics::*;
use lighting_models::raytracing::*;

const IMAGE_SIZE: [u32; 2] = [1500, 1000];
const NUM_RENDERS: u32 = 50;

fn main() {
    let (mut event_loop, vulkano_context, mut vulkano_windows, window_ids, commands_allocator, descriptor_set_allocator) = get_general_graphics_data(vec![("Scene".to_string(), IMAGE_SIZE[0] as f32, IMAGE_SIZE[1] as f32, false)], gen_swapchain_func!(Format::B8G8R8A8_UNORM));
    let mut gui = Vec::new();
    let scene_window_id = window_ids[0];

    let graphics_pipeline = RenderPassOverFrame::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        Format::B8G8R8A8_UNORM
    );
    let mut compute_pipeline = RayTracePipeine::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        IMAGE_SIZE
    );
    let mut image_combine_pipeline = ImageCombiner::new(
        &vulkano_context,
        IMAGE_SIZE,
        &commands_allocator,
        &descriptor_set_allocator
    );

    let mut camera = load_spheres_scene(&mut compute_pipeline, &vulkano_context);
    // let mut camera = load_island_scene(&mut compute_pipeline, &vulkano_context);
    // let mut camera = load_cave_scene(&mut compute_pipeline, &vulkano_context);
    camera.controllable();
    
  

    

    // let mut start_time = Instant::now();
    // let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
    // let mut before_compute = renderer.acquire().unwrap();
    // for i in 0..NUM_RENDERS {
    //     before_compute = compute_pipeline.compute(before_compute, &camera, i);
    //     image_combine_pipeline.add_image(compute_pipeline.image());
    //     start_time = Instant::now();
    // }
    // let after_combine = image_combine_pipeline.combine(before_compute);
    // let after_render = graphics_pipeline.render(after_combine, compute_pipeline.image(), renderer.swapchain_image_view());
    // renderer.present(after_render, true);
    // println!("It took {} seconds to render that image", start_time.elapsed().as_secs_f32());
    

    let mut last_frame_time = Instant::now();
    let mut offset = 0;
    loop {
        if !generic_winit_event_handling_with_camera(&mut event_loop, &mut vulkano_windows, &mut gui, (&mut camera, &scene_window_id)) {break;}

        let frame_time = last_frame_time.elapsed().as_secs_f32();
        if frame_time > 1.0 / 60.0 {
            last_frame_time = Instant::now();

            let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
            let before_compute = renderer.acquire().unwrap();
            let after_compute = compute_pipeline.compute(before_compute, &camera, offset);
            let after_diffuse = image_combine_pipeline.next_frame(compute_pipeline.image(), after_compute);
            let after_render = graphics_pipeline.render(after_diffuse, image_combine_pipeline.image(), renderer.swapchain_image_view());
            renderer.present(after_render, true);

            camera.do_move(frame_time);
            offset += 1;
            // println!("{:?}, {:?}", camera.position, camera.direction);
        }
    }

}

#[allow(dead_code)]
fn load_spheres_scene(
    pipeline: &mut RayTracePipeine,
    context: &VulkanoContext
) -> Camera {
    let spheres = vec![
        Sphere {centre: [0, -20, 0].into(), radius: 20.0, material: RayTraceMaterial {colour: [0.7, 0.1, 0.7].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}},
        Sphere {centre: [2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1, 0, 0].into(), roughness: 0.2, metalic: 1.0, ..Default::default()}},
        Sphere {centre: [-2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [0.0, 0.5, 0.5].into(), roughness: 0.2, metalic: 0.5, ..Default::default()}},
        Sphere {centre: [0, 1, 0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1.0, 0.4, 0.4].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}},
    ];

    pipeline.update_spheres(context, spheres);
    let cam = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), Some(10.0), None);
    pipeline.init_data(context, 1.0, 2.0, cam.up, 10, 0.002, 50, true);

    cam
}

#[allow(dead_code)]
fn load_island_scene(
    pipeline: &mut RayTracePipeine,
    context: &VulkanoContext
) -> Camera {
    let meshes = load_obj("assets/island.obj");
    let mesh_data = vec![
        RayTracingMesh{mesh: meshes[0].clone(), material: RayTraceMaterial{colour: [0.5; 3].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}}, // island
        RayTracingMesh{mesh: meshes[1].clone(), material: RayTraceMaterial{emmision: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 5.0].into(), colour: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0].into(), ..Default::default()}}, // water
        RayTracingMesh{mesh: meshes[2].clone(), material: RayTraceMaterial{colour: [38.0 / 255.0, 21.0 / 255.0, 5.0 / 255.0].into(), roughness: 0.5, metalic: 1.0, ..Default::default()}}, // tree
        RayTracingMesh{mesh: meshes[3].clone(), material: RayTraceMaterial{colour: [0.5, 1.0, 0.0].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}} // leaves
    ];

    pipeline.update_meshes(context, &mesh_data);
    let cam = Camera::new(Some([-4.4, 1.6, -11.9]), Some([0.5, 0.0, 1.3]), Some(10.0), None);
    pipeline.init_data(context, 1.0, 2.0, cam.up, 5, 0.001, 50, false);

    cam
}

#[allow(dead_code)]
fn load_cave_scene(
    pipeline: &mut RayTracePipeine,
    context: &VulkanoContext
) -> Camera {
    let meshes = load_obj("assets/Cave.obj");
    let mesh_data = vec![
        RayTracingMesh{mesh: meshes[4].clone(), material: RayTraceMaterial{colour: [0.5; 3].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}}, // cave
        RayTracingMesh{mesh: meshes[5].clone(), material: RayTraceMaterial{emmision: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 5.0].into(), colour: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0].into(), ..Default::default()}}, // water
        RayTracingMesh{mesh: meshes[0].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        RayTracingMesh{mesh: meshes[1].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        RayTracingMesh{mesh: meshes[2].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        RayTracingMesh{mesh: meshes[3].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
    ];

    pipeline.update_meshes(context, &mesh_data);
    let cam = Camera::new(Some([30.9, -6.5, 11.4]), Some([-1.25, 0.0, -0.6]), Some(10.0), None);
    pipeline.init_data(context, 1.0, 2.0, cam.up, 2, 0.001, 25, false);

    cam
}