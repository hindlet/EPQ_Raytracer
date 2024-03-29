use maths::{Vector3, Matrix3, Vector4};
use std::sync::Arc;
use std::collections::BTreeMap;
use std::mem::size_of;
use graphics::*;
use graphics::all_vulkano_utils::renderer::DeviceImageView;
use graphics::all_vulkano::{
    pipeline::{PipelineBindPoint, PipelineLayout, layout::{PipelineLayoutCreateInfo, PushConstantRange}, Pipeline},
    device::Queue,
    command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer},
    descriptor_set::{allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet, layout::{DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, DescriptorSetLayoutCreateInfo}},
    image::{StorageImage, ImageUsage},
    sync::GpuFuture,
    buffer::BufferContents,
    shader::ShaderStages,
};
use super::raytracing_app::RayTracerSettings;
use super::objects::*;


pub mod raytrace_shader {
    graphics::shader!{
        ty: "compute",
        path: "assets/raytracing.glsl",
        custom_derives: [Debug, Clone]
    }
}


/// The raytracing pipeline
pub struct RayTracePipeline {
    compute_queue: Arc<Queue>,
    compute_pipeline: Arc<ComputePipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    image: DeviceImageView,
    image_size: [u32; 2],

    ray_data: (Subbuffer<[raytrace_shader::Ray]>, u32),
    sphere_data: (Subbuffer<[raytrace_shader::Sphere]>, u32),
    sample_jitter: f32,
    num_samples: u32,
    max_bounces: u32,
    use_environment_lighting: bool,
    mesh_data: (Subbuffer<[raytrace_shader::Triangle]>, Subbuffer<[raytrace_shader::Mesh]>, u32)
}


impl RayTracePipeline {
    /// creates a new raytrace pipeline with the given settings
    pub fn new<T: graphics::Position + BufferContents + Copy + Clone>(
        context: &VulkanoContext,
        command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>,
        image_size: [u32; 2],
        settings: RayTracerSettings<T>
    ) -> Self {

        let pipeline = ComputePipeline::with_pipeline_layout(
            context.device().clone(),
            raytrace_shader::load(context.device().clone()).unwrap().entry_point("main").unwrap(),
            &(),
            RayTracePipeline::get_pipeline_layout(context),
            None,
        ).unwrap();
        
        let image = StorageImage::general_purpose_image_view(
            context.memory_allocator(),
            context.compute_queue().clone(),
            image_size,
            Format::R8G8B8A8_UNORM,
            ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
        ).unwrap();

        let (ray_data, num_rays, jitter) = create_ray_subbuffer(context, image_size, settings.camera_focal_length, settings.viewport_height, settings.up);
        let sphere_data = create_sphere_subbuffer(context, settings.sphere_data);
        let mesh_data = create_mesh_subbuffer(context, &settings.mesh_data);
        

        RayTracePipeline {
            compute_queue: context.graphics_queue().clone(),
            compute_pipeline: pipeline,
            command_buffer_allocator: command_buffer_allocator.clone(),
            descriptor_set_allocator: descriptor_set_allocator.clone(),
            image: image,
            image_size: image_size,

            num_samples: settings.num_samples.max(1),
            max_bounces: settings.max_bounces.max(0),
            use_environment_lighting: settings.use_environment_lighting,
            sample_jitter: settings.sample_jitter.unwrap_or(jitter),

            ray_data: (ray_data, num_rays),
            sphere_data: sphere_data,
            mesh_data: mesh_data,
        }
    }

    /// return the pipeline layout, maually adjusted
    fn get_pipeline_layout(
        context: &VulkanoContext
    ) -> Arc<PipelineLayout> {

        let mut bindings = BTreeMap::new();

        bindings.insert(0, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageImage));
        bindings.insert(1, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer));
        bindings.insert(2, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer));
        bindings.insert(3, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer));
        bindings.insert(4, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer));

        for binding in bindings.iter_mut() {
            binding.1.stages = ShaderStages::COMPUTE;
        }

        let set_layout = DescriptorSetLayout::new(
            context.device().clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: bindings,
                push_descriptor: false,
                ..Default::default()
            }
        ).unwrap();

        let push_const_size = 
            size_of::<f32>() * 4 + // cam poss
            size_of::<f32>() * 16 + // cam allignment mat
            size_of::<i32>() + // num_rays;
            size_of::<i32>() + // num_spheres;
            size_of::<i32>() + // num_meshes
            size_of::<i32>() + // num_samples;
            size_of::<f32>() + // jitter_size;
            size_of::<i32>() + // max_bounces;
            size_of::<u32>() + // use_environment_light
            size_of::<u32>() + // rng_offset
            size_of::<u32>() + // init
            size_of::<u32>() + // width
            size_of::<u32>() // height
        ;


        PipelineLayout::new(context.device().clone(),
            PipelineLayoutCreateInfo {
                set_layouts: vec![set_layout],
                push_constant_ranges: vec![PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    offset: 0,
                    size: push_const_size as u32
                }],
                ..Default::default()
            }
        ).unwrap()
    }

    /// returns the pipeline image
    pub fn image(&self) -> DeviceImageView {
        self.image.clone()
    }


    /// next pass of raytracing
    pub fn compute(
        &mut self,
        before_future: Box<dyn GpuFuture>,
        camera: &Camera,
        rng_offset: u32,
    ) -> Box<dyn GpuFuture> {

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        self.dispatch(&mut builder, camera, rng_offset, false);


        let command_buffer = builder.build().unwrap();
        let after_future = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

            
        after_future.boxed()
    }

    // call the init to fill the screen with black
    pub fn init(
        &self,
        before_future: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        self.dispatch(&mut builder, &Camera::new(None, None, None, None), 0, true);


        let command_buffer = builder.build().unwrap();
        let after_future = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();


        after_future.boxed()
    }

    // send data to the gpu
    fn dispatch(
        &self,
        builder: &mut AutoCommandBufferBuilder<
        PrimaryAutoCommandBuffer,
        Arc<StandardCommandBufferAllocator>>,
        camera: &Camera,
        rng_offset: u32,
        init: bool
    ) {
        let pipeline_layout = self.compute_pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.image.clone()),
                WriteDescriptorSet::buffer(1, self.ray_data.0.clone()),
                WriteDescriptorSet::buffer(2, self.sphere_data.0.clone()),
                WriteDescriptorSet::buffer(3, self.mesh_data.0.clone()),
                WriteDescriptorSet::buffer(4, self.mesh_data.1.clone())
            ],
        )
        .unwrap();
        
        let to_process_x = (self.image_size[0] - 1) / 32 + 1;
        let to_process_y = (self.image_size[1] - 1) / 32 + 1;

        let push_constants = raytrace_shader::PushConstants {
            cam_pos: camera.position.extend().into(),
            cam_alignment_mat: self.get_view_matrix(camera),
            num_rays: self.ray_data.1 as i32,
            num_spheres: self.sphere_data.1 as i32,
            num_meshes: self.mesh_data.2 as i32,
            num_samples: self.num_samples as i32,
            jitter_size: self.sample_jitter,
            max_bounces: self.max_bounces as i32,
            use_environment_light: self.use_environment_lighting as u32,
            rng_offset: rng_offset,
            init: init as u32,
            width: self.image_size[0],
            height: self.image_size[1]
        };


        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([to_process_x, to_process_y, 1])
            .unwrap();
    }

    /// https://math.stackexchange.com/questions/2546457/plane-vector-rotation
    fn get_view_matrix(
        &self,
        camera: &Camera,
    ) -> [[f32; 4]; 4]{
        let new_x = camera.direction.normalised();
        let new_z = -camera.direction.cross(camera.up).normalised();
        let new_y = -new_x.cross(new_z).normalised();

        
        let allignment_mat = Matrix3::from_columns(new_x, new_y, new_z).transposed();
        [
            allignment_mat.x.extend().into(),
            allignment_mat.y.extend().into(),
            allignment_mat.z.extend().into(),
            Vector4::W.into(),
        ]
    }
}

// creates the rays sent to the shader, they are centered around (1, 0, 0) and get transformed there
fn create_ray_subbuffer(
    context: &VulkanoContext,
    image_size: [u32; 2],
    camera_focal_length: f32,
    viewport_height: f32,
    up: impl Into<Vector3>,
) -> (Subbuffer<[raytrace_shader::Ray]>, u32, f32) {

    // zero length protection
    if image_size[0] == 0 || image_size[1] == 0 {
        let null_ray = vec![raytrace_shader::Ray {
            sample_centre: [0.0, 0.0, 0.0, 0.0],
        }];
        return (create_shader_data_buffer(null_ray, context, BufferType::Storage), 0, 0.0);
    }



    let viewport_width = viewport_height * (image_size[0] as f32 / image_size[1] as f32);

    let viewport_x = up.into().cross(Vector3::X).normalised();
    let viewport_y = viewport_x.cross(Vector3::X).normalised();
    let viewport_upper_left = Vector3::ZERO + Vector3::X * camera_focal_length - (viewport_x * viewport_width + viewport_y * viewport_height) * 0.5;

    let pixel_x: Vector3 = viewport_x * viewport_width / image_size[0] as f32;
    let pixel_y: Vector3 = viewport_y * viewport_height / image_size[1] as f32;

    let first_ray = viewport_upper_left + (pixel_x + pixel_y) * 0.5;
    let mut ray_centres: Vec<[f32; 4]> = Vec::new();

    for y in 0..image_size[1] {
        for x in 0..image_size[0] {
            let ray_pos = first_ray + pixel_x * x as f32 + pixel_y * y as f32;
            ray_centres.push(
                ray_pos.extend().into()
            );
        }
    }


    let mut rays: Vec<raytrace_shader::Ray> = Vec::new();
    for centre in ray_centres {
        rays.push(raytrace_shader::Ray {
            sample_centre: centre,
        });
    }

    let num_rays = rays.len() as u32;
    (create_shader_data_buffer(rays, context, BufferType::Storage), num_rays, pixel_x.magnitude().max(pixel_y.magnitude()) * 0.5)
}


/// transformes list of spheres to subbuffer of raytrace spheres
fn create_sphere_subbuffer(
    context: &VulkanoContext,
    sphere_data: Vec<Sphere>
) -> (Subbuffer<[raytrace_shader::Sphere]>, u32) {

    // zero length protection
    if sphere_data.len() == 0 {
        return (create_shader_data_buffer(vec![get_null_sphere().into()], context, BufferType::Storage), 0);
    }

    let mut spheres: Vec<raytrace_shader::Sphere> = Vec::new();

    for sphere in sphere_data.iter() {
        spheres.push(sphere.clone().into());
    }

    let num_spheres = spheres.len() as u32;
    (create_shader_data_buffer(spheres, context, BufferType::Storage), num_spheres)
}

/// transformes list of meshes to subbuffer of raytrace meshes
fn create_mesh_subbuffer<T: graphics::Position + BufferContents + Copy + Clone>(
    context: &VulkanoContext,
    meshes: &Vec<RayTracingMesh<T>>,
) -> (Subbuffer<[raytrace_shader::Triangle]>, Subbuffer<[raytrace_shader::Mesh]>, u32) {

    // zero length protection
    let (tris, mesh_data) = if meshes.len() == 0 {transform_meshes(&vec![get_null_mesh()])} else {transform_meshes(meshes)};

    let tri_buffer = create_shader_data_buffer(tris, context, BufferType::Storage);
    let mesh_buffer = create_shader_data_buffer(mesh_data, context, BufferType::Storage);
    (tri_buffer, mesh_buffer, meshes.len() as u32)
}

/// transform meshes into triangles and mesh info
fn transform_meshes<T: graphics::Position + BufferContents + Copy + Clone>(
    meshes: &Vec<RayTracingMesh<T>>,
) -> (Vec<raytrace_shader::Triangle>, Vec<raytrace_shader::Mesh>){

    let mut tri_count = 0;
    let mut tris: Vec<raytrace_shader::Triangle> = Vec::new();
    let mut mesh_data: Vec<raytrace_shader::Mesh> = Vec::new();
    for mesh in meshes.iter() {

        let mat = mesh.material.clone();
        let mesh = mesh.mesh.clone();
        let (mut min_x, mut min_y, mut min_z) = (f32::MAX, f32::MAX, f32::MAX);
        let (mut max_x, mut max_y, mut max_z) = (f32::MIN, f32::MIN, f32::MIN);
        

        for i in (0..mesh.indices.len()).step_by(3) {
            let a: Vector3 = mesh.vertices[mesh.indices[i + 0] as usize].pos().into();
            let b: Vector3 = mesh.vertices[mesh.indices[i + 1] as usize].pos().into();
            let c: Vector3 = mesh.vertices[mesh.indices[i + 2] as usize].pos().into();
            let edge_one = b - a;
            let edge_two = c - a;
            let norm = edge_one.cross(edge_two);

            min_x = min_x.min(a.x.min(b.x.min(c.x)));
            min_y = min_y.min(a.y.min(b.y.min(c.y)));
            min_z = min_z.min(a.z.min(b.z.min(c.z)));

            max_x = max_x.max(a.x.max(b.x.max(c.x)));
            max_y = max_y.max(a.y.max(b.y.max(c.y)));
            max_z = max_z.max(a.z.max(b.z.max(c.z)));

            tris.push(raytrace_shader::Triangle {
                a: a.extend().into(),
                edge_one: edge_one.extend().into(),
                edge_two: edge_two.extend().into(),
                normal: norm.extend().into()
            })
        }

        let num_tris = mesh.indices.len() as u32 / 3;
        mesh_data.push(raytrace_shader::Mesh {
            first_index: tri_count,
            len: num_tris,
            material: mat,
            min_point: [min_x, min_y, min_z],
            max_point: [max_x, max_y, max_z]
        });
        tri_count += num_tris;
    }

    (tris, mesh_data)
}