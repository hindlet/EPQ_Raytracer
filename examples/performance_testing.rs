use std::time::Instant;
use graphics::*;
use maths::Vector3;
use lighting_models::{gen_icosphere, blinn_phong::*};

const AMBIENT_STRENGTH: f32 = 0.1 * 1.0;
const DIFFUSE_STRENGTH: f32 = 0.5 * 1.0;
const SPECULAR_STRENGTH: f32 = 2.0 * 1.0;
const NUM_SAMPLES: usize = 5;
const NUM_SPHERES: usize = 10;


fn main() {

    let (_, vulkano_context, mut vulkano_windows, window_ids, commands_allocator, descriptor_set_allocator) = get_general_graphics_data(vec![("Scene".to_string(), 1500.0, 1000.0, false)], gen_swapchain_func!(Format::B8G8R8A8_SRGB));
    let uniform_allocator = create_uniform_buffer_allocator(vulkano_context.memory_allocator());

    for i in 1..=NUM_SPHERES {

        let mut total_mesh = Mesh::EMPTY;
        let sphere = gen_icosphere(20.0, Vector3::Y * -20.0, [1.0, 1.0, 1.0, 1.0], 5);

        for _ in 0..(i * 100) {
            total_mesh.add(sphere.clone());
        }
        
        let lights = vec![
            Light{pos: [3, 2, 0].into(), strength: 0.3, colour: [1, 1, 1].into()}, // white
            Light{pos: [0, 3, 0].into(), strength: 0.5, colour: [1, 0, 1].into()}, // purple
            Light{pos: [-3, 5, 0].into(), strength: 0.8, colour: [0, 1, 1].into()}, // whatever blue + green is
            Light{pos: [3.75, 0.0, 0.0].into(), strength: 0.7, colour: [0, 1, 0].into()}, // green
        ];

        let light_buffer = get_light_buffer(&uniform_allocator, lights);
        
        let (total_vertices, total_normals, total_indices) = total_mesh.components();
        let num_tris = total_indices.len() / 3;

        let vertex_buffer = create_shader_data_buffer(total_vertices, &vulkano_context, BufferType::Vertex);
        let normal_buffer = create_shader_data_buffer(total_normals, &vulkano_context, BufferType::Normal);
        let index_buffer = create_shader_data_buffer(total_indices, &vulkano_context, BufferType::Index);

        let camera = Camera::new(Some([2.0, 2.0, -5.0]), Some([-0.35, -0.35, 0.87]), Some(10.0), None);

        let scene_window_id = window_ids[0];


        let mut pipeline = BlinnPhongPipeline::new(
            &vulkano_context,
            &commands_allocator,
            &descriptor_set_allocator,
            Some(SampleCount::Sample4),
        );

        let uniforms = get_uniforms(vulkano_windows.get_renderer_mut(scene_window_id).unwrap().swapchain_image_size(), &uniform_allocator, &camera, AMBIENT_STRENGTH, DIFFUSE_STRENGTH, SPECULAR_STRENGTH);


        let mut total_time = 0.0;
        let mut draw_time = Instant::now();
        for _ in 0..NUM_SAMPLES {
            let renderer = vulkano_windows.get_renderer_mut(scene_window_id).unwrap();
            let before_future = renderer.acquire().unwrap();
            let after_future = pipeline.draw(before_future, renderer.swapchain_image_view(), &vertex_buffer, &normal_buffer, &index_buffer, &uniforms, &light_buffer);
            renderer.present(after_future, true);
            total_time += draw_time.elapsed().as_secs_f32();
            draw_time = Instant::now();
        }
        println!("It took {} seconds to to render {} triangles ({} spheres)", total_time / NUM_SAMPLES as f32, num_tris, i * 100);
    }



}

