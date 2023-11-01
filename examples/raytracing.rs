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
    // let mut app = load_box_scene();
    // let mut app = load_cube_scene();
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
    
    // compute_then_render(&mut app, 0.0);

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
        Sphere {
            centre: [0.0, -20.0, 0.0], 
            radius: 20.0, 
            material: LambertianMaterial {
                colour: [0.7, 0.1, 0.7].into(),
            }.to_mat()
        },
        Sphere {
            centre: [2.5, 0.75, 0.0], 
            radius: 1.0, 
            material: MetalMaterial {
                colour: [1.0, 0.0, 0.0].into(),
                smoothness: 1.0,
                fuzz: 0.0
            }.to_mat()
        },
        Sphere {
            centre: [-2.5, 0.75, 0.0], 
            radius: 1.0, 
            material: MetalMaterial {
                colour: [0.0, 0.5, 0.5].into(),
                smoothness: 1.0,
                fuzz: 0.0,
            }.to_mat()
        },
        Sphere {
            centre: [0.0, 1.0, 0.0], 
            radius: 1.0,
            material: MetalMaterial {
                colour: [1.0, 0.4, 0.4].into(),
                smoothness: 1.0,
                fuzz: 0.0,
            }.to_mat()
        },
    ];

    let cam = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), Some(10.0), None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            sample_settings: (0.005, 2, 25, true),
            sphere_data: spheres,
            mesh_data: Vec::new(),
            camera_focal_length: 1.0,
            viewport_height: 2.0,
            up: up.into()
        }
    )
}


#[allow(dead_code)]
fn load_box_scene() -> RayTracingApp<PositionVertex>{
    let meshes = load_obj("assets/box.obj");
    let mesh_data = vec![
        RayTracingMesh{ // floor
            mesh: meshes[1].clone(),
            material: LambertianMaterial{
                colour: [1.0, 1.0, 1.0],
            }.to_mat()
        },
        RayTracingMesh{ // wall
            mesh: meshes[3].clone(),
            material: LambertianMaterial{
                colour: [166.0 / 255.0, 45.0 / 255.0, 23.0 / 255.0],
            }.to_mat()
        },
        RayTracingMesh{ // wall
            mesh: meshes[4].clone(),
            material: LambertianMaterial{
                colour: [19.0 / 255.0, 133.0 / 255.0, 34.0 / 255.0],
            }.to_mat()
        },
        RayTracingMesh{ // wall
            mesh: meshes[5].clone(),
            material: LambertianMaterial{
                colour: [28.0 / 255.0, 83.0 / 255.0, 112.0 / 255.0],
            }.to_mat()
        },
        RayTracingMesh{ // ceiling
            mesh: meshes[2].clone(),
            material: MetalMaterial{
                colour: [1.0, 1.0, 1.0],
                smoothness: 1.0,
                fuzz: 0.5
            }.to_mat()
        },
        RayTracingMesh{ // light
            mesh: meshes[0].clone(),
            material: LightMaterial {
                emission: [1.0, 1.0, 1.0, 5.0]
            }.to_mat()
        }
    ];

    let sphere_data = vec![
        // Sphere{
        //     centre: [0.0, 1.0, 0.0].into(),
        //     radius: 1.0,
        //     material: MetalMaterial {
        //         smoothness: 1.0,
        //         fuzz: 0.0,
        //         colour: [1.0, 1.0, 1.0].into(),
        //     }.to_mat()
        // },
        // Sphere {
        //     centre: [0.0, 2.5, 0.0],
        //     radius: 0.5,
        //     material: InvisLightMaterial {
        //         emission: [1.0, 1.0, 1.0, 5.0]
        //     }.to_mat()
        // }
    ];

    let cam = Camera::new(Some([8.0, 1.5, 0.0]), Some([-1.0, 0.0, 0.0]), None, None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            sample_settings: (0.005, 1, 50, false),
            sphere_data: sphere_data,
            mesh_data: mesh_data,
            camera_focal_length: 1.0,
            viewport_height: 2.0,
            up: up.into()
        }
    )

}

#[allow(dead_code)]
fn load_cube_scene() -> RayTracingApp<PositionVertex>{
    let meshes = load_obj("assets/Cube.obj");
    let mesh_data = vec![
        RayTracingMesh{
            mesh: meshes[0].clone(),
            material: MetalMaterial{
                colour: [1.0, 0.0, 1.0],
                smoothness: 1.0,
                fuzz: 0.0
            }.to_mat()
        },
    ];

    let sphere_data = vec![
        Sphere{
            centre: [0.0, 0.0, 0.0].into(),
            radius: 1.0,
            material: MetalMaterial {
                smoothness: 1.0,
                fuzz: 0.0,
                colour: [1.0, 1.0, 1.0].into(),
            }.to_mat()
        },
    ];


    let cam = Camera::new(Some([3.0, 1.5, 0.0]), Some([-1.0, 0.0, 0.0]), None, None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            sample_settings: (0.005, 10, 50, true),
            sphere_data: sphere_data,
            mesh_data: mesh_data,
            camera_focal_length: 1.0,
            viewport_height: 2.0,
            up: up.into()
        }
    )
}