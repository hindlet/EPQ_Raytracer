use std::time::Instant;
use graphics::*;
use lighting_models::raytracing::*;

const IMAGE_SIZE: [u32; 2] = [1080, 720];
const TARGET_FPS: f32 = 60.0;
const TARGET_FRAME_TIME: f32 = 1.0 / TARGET_FPS;
const NUM_RENDERS: usize = 50;
const REALTIME: bool = true;

fn main() {
    let mut event_loop = EventLoop::new();


    let mut app = load_spheres_scene();
    // let mut app = load_box_scene();
    // let mut app = load_cube_scene();
    // app.camera.controllable();
    
  
    app.open(&event_loop, IMAGE_SIZE);
    


    if !REALTIME {
        compute_n_then_render(&mut app, NUM_RENDERS);
    }

    let mut last_frame_time = Instant::now();
    loop {
        if !handle_events(&mut app, &mut event_loop) {break;}

        if !REALTIME{continue;}
        let frame_time = last_frame_time.elapsed().as_secs_f32();
        if frame_time >= TARGET_FRAME_TIME {
            last_frame_time = Instant::now();
            
            compute_then_render(&mut app, frame_time);
            // println!("{:?}, {:?}", camera.position, camera.direction);
            // if last_frame_time.elapsed().as_secs_f32() > TARGET_FRAME_TIME {println!("Slow frame")}
        }
    }

}

#[allow(dead_code)]
fn load_spheres_scene() -> RayTracingApp<PositionVertex>{
    let spheres = vec![
        Sphere {
            centre: [0.0, -100.0, 0.0], 
            radius: 100.0, 
            material: LambertianMaterial {
                colour: [0.5, 0.5, 0.5].into(),
            }.into()
        },
        Sphere {
            centre: [2.5, 0.75, 0.0], 
            radius: 1.0, 
            material: MetalMaterial {
                colour: [0.2, 0.2, 1.0].into(),
                smoothness: 1.0,
                fuzz: 0.1
            }.into()
        },
        Sphere {
            centre: [-2.5, 0.75, 0.0], 
            radius: 1.0, 
            material: MetalMaterial {
                colour: [1.0, 0.2, 0.2].into(),
                smoothness: 1.0,
                fuzz: 0.1,
            }.into()
        },
        Sphere {
            centre: [0.0, 1.0, 0.0], 
            radius: 1.0,
            material: MetalMaterial {
                colour: [0.2, 1.0, 0.2].into(),
                smoothness: 1.0,
                fuzz: 0.1,
            }.into()
        },

        Sphere {
            centre: [500.0, 100.0, 500.0],
            radius: 250.0,
            material: InvisLightMaterial {
                emission: [0.6, 0.6, 1.0, 25.0]
            }.into()
        },
    ];

    let view_dir = [-0.35, -0.35, 0.87];
    // println!("{:?}", maths::Vector3::direction_to_euler_angles(view_dir));
    let cam = Camera::new(Some([2.0, 2.0, -5.0]), Some(view_dir), Some(10.0), None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            num_samples: 25,
            max_bounces: 50,
            use_environment_lighting: false,
            sample_jitter: None,
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
            mesh: meshes[0].clone(),
            material: CustomMaterial {
                colour: [1.0, 1.0, 1.0],
                smoothness: 0.7,
                specular_probability: 0.5,
                ..Default::default()
            }.into()
        },
        RayTracingMesh{ // Left Wall
            mesh: meshes[4].clone(),
            material: CustomMaterial {
                colour: [166.0 / 255.0, 45.0 / 255.0, 23.0 / 255.0],
                smoothness: 0.7,
                specular_probability: 0.5,
                ..Default::default()
            }.into()
        },
        RayTracingMesh{ // Right Wall
            mesh: meshes[3].clone(),
            material: CustomMaterial {
                colour: [19.0 / 255.0, 133.0 / 255.0, 34.0 / 255.0],
                smoothness: 0.7,
                specular_probability: 0.5,
                ..Default::default()
            }.into()
        },
        RayTracingMesh{ // Back Wall
            mesh: meshes[1].clone(),
            material: CustomMaterial {
                colour: [1.0; 3],
                smoothness: 0.7,
                specular_probability: 0.5,
                ..Default::default()
            }.into()
        },
        RayTracingMesh{ // ceiling
            mesh: meshes[5].clone(),
            material: CustomMaterial {
                colour: [1.0; 3],
                smoothness: 0.7,
                specular_probability: 0.5,
                ..Default::default()
            }.into()
        },
        RayTracingMesh{ // Front Wall
            mesh: meshes[2].clone(),
            material: CustomMaterial {
                colour: [1.0; 3],
                smoothness: 0.7,
                specular_probability: 0.5,
                ..Default::default()
            }.into()
        },
        RayTracingMesh{ // light
            mesh: meshes[6].clone(),
            material: LightMaterial {
                emission: [1.0, 1.0, 1.0, 5.0]
            }.into()
        }
    ];

    let sphere_data = vec![
        Sphere{
            centre: [-0.5, 0.5, 0.0].into(),
            radius: 0.5,
            material: MetalMaterial {
                smoothness: 1.0,
                fuzz: 0.0,
                colour: [1.0, 1.0, 1.0].into(),
            }.into()
        },
        Sphere{
            centre: [0.5, 0.5, 0.0].into(),
            radius: 0.5,
            material: MetalMaterial {
                smoothness: 1.0,
                fuzz: 0.0,
                colour: [1.0, 1.0, 1.0].into(),
            }.into()
        },
    ];

    let cam = Camera::new(Some([1.5, 1.0, 0.0]), Some([-1.0, 0.0, 0.0]), None, None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            num_samples: 5,
            max_bounces: 50,
            use_environment_lighting: false,
            sample_jitter: Some(0.001),
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
                colour: [0.7, 0.7, 0.7],
                smoothness: 1.0,
                fuzz: 0.0
            }.into()
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
            }.into()
        },
    ];


    let cam = Camera::new(Some([5.0, 2.0, 0.0]), Some([-1.0, -0.2, 0.0]), None, None);
    let up = cam.up;
    RayTracingApp::new(
        cam,
        RayTracerSettings {
            num_samples: 10,
            max_bounces: 50,
            use_environment_lighting: true,
            sample_jitter: None,
            sphere_data: sphere_data,
            mesh_data: mesh_data,
            camera_focal_length: 1.0,
            viewport_height: 2.0,
            up: up.into()
        }
    )
}