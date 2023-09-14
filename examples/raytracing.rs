use std::time::Instant;
use graphics::*;
use lighting_models::raytracing::*;

const IMAGE_SIZE: [u32; 2] = [1000, 750];

fn main() {
    let (mut event_loop, vulkano_context, mut vulkano_windows, window_ids, commands_allocator, descriptor_set_allocator) = get_general_graphics_data(vec![("Scene".to_string(), IMAGE_SIZE[0] as f32, IMAGE_SIZE[1] as f32, false)], gen_swapchain_func!(Format::B8G8R8A8_UNORM));
    let mut gui = Vec::new();

    let mut camera = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), None, None);

    let mut last_frame_time = Instant::now();

    let scene_window_id = window_ids[0];
    

    let mut compute_pipeline = RayTracePipeine::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        IMAGE_SIZE
    );
    compute_pipeline.init_data(&vulkano_context, 1.0, 2.0, camera.up);
    compute_pipeline.update_spheres(&vulkano_context, vec![
        ([0.0, -20.0, 0.0], 20.0),
        ([2.5, 0.75, 0.0], 1.0),
        ([-2.5, 0.75, 0.0], 1.0),
        ([0.0, 1.0, 0.0], 1.0),
    ]);
    

    let graphics_pipeline = RenderPassOverFrame::new(
        &vulkano_context,
        &commands_allocator,
        &descriptor_set_allocator,
        Format::B8G8R8A8_UNORM
    );

    loop {
        if !generic_winit_event_handling_with_camera(&mut event_loop, &mut vulkano_windows, &mut gui, (&mut camera, &scene_window_id)) {break;}

        let frame_time = last_frame_time.elapsed().as_secs_f32();
        if frame_time > 1.0 / 60.0 {
            last_frame_time = Instant::now();

            let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
            let before_compute = renderer.acquire().unwrap();
            let after_compute = compute_pipeline.compute(before_compute, &camera);
            let after_render = graphics_pipeline.render(after_compute, compute_pipeline.image(), renderer.swapchain_image_view());
            renderer.present(after_render, true);

            camera.do_move(frame_time);
        }

    }
}