use std::time::Instant;
use graphics::*;
use lighting_models::raytracing::*;

const IMAGE_SIZE: [u32; 2] = [1080, 720];
const NUM_RENDERS: u32 = 50;
const TARGET_FPS: f32 = 30.0;
const TARGET_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

fn main() {
    let mut event_loop = EventLoop::new();


    let mut app = load_spheres_scene();
    // let mut app = load_island_scene();
    // let mut app = load_cave_scene();
    app.camera.controllable();
    
  
    app.open(&event_loop, IMAGE_SIZE);
    

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
    loop {
        if !handle_events(&mut app, &mut event_loop) {break;}

        let frame_time = last_frame_time.elapsed().as_secs_f32();
        if frame_time >= TARGET_FRAME_TIME {
            last_frame_time = Instant::now();
            
            compute_then_render(&mut app, frame_time);
            // println!("{:?}, {:?}", camera.position, camera.direction);
        }
    }

}

#[allow(dead_code)]
fn load_spheres_scene() -> RayTracingApp<PositionVertex>{
    let spheres = vec![
        Sphere {centre: [0, -20, 0].into(), radius: 20.0, material: RayTraceMaterial {colour: [0.7, 0.1, 0.7].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}},
        Sphere {centre: [2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1, 0, 0].into(), roughness: 0.2, metalic: 1.0, ..Default::default()}},
        Sphere {centre: [-2.5, 0.75, 0.0].into(), radius: 1.0, material: RayTraceMaterial {colour: [0.0, 0.5, 0.5].into(), roughness: 0.2, metalic: 0.5, ..Default::default()}},
        Sphere {centre: [0, 1, 0].into(), radius: 1.0, material: RayTraceMaterial {colour: [1.0, 0.4, 0.4].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}},
        Sphere {centre: [0, 15, 0].into(), radius: 5.0, material: RayTraceMaterial {emmision: [1.0, 1.0, 1.0, 5.0].into(), metalic: 1.0, roughness: 0.0, colour: [1; 3].into()}}
    ];

    let cam = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), Some(10.0), None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            sample_settings: (0.005, 1, 50, false),
            sphere_data: spheres,
            mesh_data: Vec::new(),
            camera_focal_length: 1.0,
            viewport_height: 2.0,
            up: up.into()
        }
    )
}

#[allow(dead_code)]
fn load_island_scene() -> RayTracingApp<PositionVertex>{
    let meshes = load_obj("assets/island.obj");
    let mesh_data = vec![
        RayTracingMesh{mesh: meshes[0].clone(), material: RayTraceMaterial{colour: [0.5; 3].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}}, // island
        RayTracingMesh{mesh: meshes[1].clone(), material: RayTraceMaterial{emmision: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 5.0].into(), colour: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0].into(), ..Default::default()}}, // water
        RayTracingMesh{mesh: meshes[2].clone(), material: RayTraceMaterial{colour: [38.0 / 255.0, 21.0 / 255.0, 5.0 / 255.0].into(), roughness: 0.5, metalic: 1.0, ..Default::default()}}, // tree
        RayTracingMesh{mesh: meshes[3].clone(), material: RayTraceMaterial{colour: [0.5, 1.0, 0.0].into(), roughness: 0.0, metalic: 1.0, ..Default::default()}} // leaves
    ];

    let cam = Camera::new(Some([-4.4, 1.6, -11.9]), Some([0.5, 0.0, 1.3]), Some(10.0), None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            sample_settings: (0.001, 1, 50, false),
            sphere_data: Vec::new(),
            mesh_data: mesh_data,
            camera_focal_length: 1.0,
            viewport_height: 2.0,
            up: up.into()
        }
    )

}

#[allow(dead_code)]
fn load_cave_scene() -> RayTracingApp<PositionVertex>{
    let meshes = load_obj("assets/Cave.obj");
    let mesh_data = vec![
        RayTracingMesh{mesh: meshes[4].clone(), material: RayTraceMaterial{colour: [0.5; 3].into(), roughness: 1.0, metalic: 0.0, ..Default::default()}}, // cave
        RayTracingMesh{mesh: meshes[5].clone(), material: RayTraceMaterial{emmision: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0, 5.0].into(), colour: [29.0 / 255.0, 154.0 / 255.0, 163.0 / 255.0].into(), ..Default::default()}}, // water
        RayTracingMesh{mesh: meshes[0].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        RayTracingMesh{mesh: meshes[1].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        RayTracingMesh{mesh: meshes[2].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
        RayTracingMesh{mesh: meshes[3].clone(), material: RayTraceMaterial{emmision: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0, 2.0].into(), colour: [194.0 / 255.0, 57.0 / 255.0, 212.0 / 255.0].into(), ..Default::default()}}, // crystal
    ];

    let cam = Camera::new(Some([30.9, -6.5, 11.4]), Some([-1.25, 0.0, -0.6]), Some(10.0), None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            sample_settings: (0.001, 1, 25, false),
            sphere_data: Vec::new(),
            mesh_data: mesh_data,
            camera_focal_length: 1.0,
            viewport_height: 2.0,
            up: up.into(),
        }
    )

}