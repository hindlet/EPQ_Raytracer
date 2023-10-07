use maths::{Vector3, Matrix3, Vector4};
use super::*;

mod raytrace_shader {
    graphics::shader!{
        ty: "compute",
        path: "assets/raytracing.glsl",
    }
}


const WORKGROUP_SIZE: u32 = 64;

#[derive(Clone, Copy, Debug)]
pub struct RayTraceMaterial {
    pub colour: Vector3,
    pub roughness: f32,
    pub metalic: f32,
    pub emmision: Vector4
}

impl Into<raytrace_shader::RayTracingMaterial> for RayTraceMaterial {
    fn into(self) -> raytrace_shader::RayTracingMaterial {
        let normal_colour = self.colour.normalised();
        raytrace_shader::RayTracingMaterial {
            colour: [normal_colour.x, normal_colour.y, normal_colour.z, 1.0],
            emission: self.emmision.into(),
            settings: [self.roughness.clamp(0.0, 1.0), self.metalic.clamp(0.0, 1.0), 0.0, 0.0]
        }
    }
}

impl Default for RayTraceMaterial {
    fn default() -> Self {
        RayTraceMaterial {
            colour: Vector3::ZERO,
            emmision: Vector4::ZERO,
            roughness: 0.5,
            metalic: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sphere {
    pub centre: Vector3,
    pub radius: f32,
    pub material: RayTraceMaterial
}

impl Into<raytrace_shader::Sphere> for Sphere {
    fn into(self) -> raytrace_shader::Sphere {
        raytrace_shader::Sphere {
            centre: self.centre.into(),
            radius: self.radius,
            material: self.material.into()
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere {
            centre: Vector3::ZERO,
            radius: 0.0,
            material: RayTraceMaterial::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct RayTracingMesh<T: graphics::Position + BufferContents + Copy + Clone> {
    pub mesh: Mesh<T>,
    pub material: RayTraceMaterial
}

fn get_null_mesh() -> RayTracingMesh<PositionVertex> {
    let mut mesh = Mesh::new(vec![PositionVertex{position: [0.0; 3]}], vec![0, 0, 0]);
    mesh.set_normals(vec![Normal{normal: [1.0; 3]}]);
    RayTracingMesh {
        mesh: mesh,
        material: RayTraceMaterial::default()
    }
}



pub struct RayTracePipeine {
    compute_queue: Arc<Queue>,
    compute_pipeline: Arc<ComputePipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    image: DeviceImageView,
    image_size: [u32; 2],

    ray_data: (Subbuffer<[raytrace_shader::Ray]>, u32),
    sphere_data: (Subbuffer<[raytrace_shader::Sphere]>, u32),
    sample_data: (f32, u32, u32, bool), // jitter_size, num_samples, max_bounces, use_environment_lighting
    mesh_data: (Subbuffer<[raytrace_shader::Triangle]>, Subbuffer<[raytrace_shader::Mesh]>, u32)
}


impl RayTracePipeine {
    pub fn new(
        context: &VulkanoContext,
        command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>,
        image_size: [u32; 2],
    ) -> Self {

        let pipeline = ComputePipeline::with_pipeline_layout(
            context.device().clone(),
            raytrace_shader::load(context.device().clone()).unwrap().entry_point("main").unwrap(),
            &(),
            RayTracePipeine::get_pipeline_layout(context),
            None,
        ).unwrap();
        
        let image = StorageImage::general_purpose_image_view(
            context.memory_allocator(),
            context.compute_queue().clone(),
            image_size,
            Format::R8G8B8A8_UNORM,
            ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
        ).unwrap();

        let null_ray = raytrace_shader::Ray {
            sample_centre: [0.0, 0.0, 0.0, 0.0],
            img_pos: [0.0, 0.0, 0.0, 0.0]
        };
        let null_sphere: raytrace_shader::Sphere = Sphere::default().into();
        let (null_tris, null_meshes) = transform_meshes(context, &vec![get_null_mesh()]); //CAUSES CRASH BC EMPTY BUFFER
        

        RayTracePipeine {
            compute_queue: context.graphics_queue().clone(),
            compute_pipeline: pipeline,
            command_buffer_allocator: command_buffer_allocator.clone(),
            descriptor_set_allocator: descriptor_set_allocator.clone(),
            image: image,
            image_size: image_size,

            ray_data: (create_shader_data_buffer(vec![null_ray], context, BufferType::Storage), 0),
            sphere_data: (create_shader_data_buffer(vec![null_sphere], context, BufferType::Storage), 0),
            sample_data: (0.0, 1, 1, true),
            mesh_data: (null_tris, null_meshes, 0),
        }
    }

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
            size_of::<u32>() // use_environment_light
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


    pub fn image(&self) -> DeviceImageView {
        self.image.clone()
    }

    /// TODO: Have rays relative to dir (1, 0, 0) and then transform in shader from cam dir
    pub fn init_data(
        &mut self,
        context: &VulkanoContext,
        camera_focal_length: f32,
        viewport_height: f32,
        up: impl Into<Vector3>,
        samples_per_pixel: u32,
        jitter_size: f32,
        max_bounces: u32,
        use_environment_lighting: bool
    ) {
        let viewport_width = viewport_height * (self.image_size[0] as f32 / self.image_size[1] as f32);

        let viewport_x = up.into().cross(Vector3::X).normalised();
        let viewport_y = viewport_x.cross(Vector3::X).normalised();
        let viewport_upper_left = Vector3::ZERO + Vector3::X * camera_focal_length - (viewport_x * viewport_width - viewport_y * viewport_height) * 0.5;

        let pixel_x = viewport_x * viewport_width / self.image_size[0] as f32;
        let pixel_y = viewport_y * viewport_height / self.image_size[1] as f32;

        let first_ray = viewport_upper_left + (pixel_x - pixel_y) * 0.5;
        let mut ray_data: Vec<([f32; 4], [f32; 4])> = Vec::new();

        for y in 0..self.image_size[1] {
            for x in 0..self.image_size[0] {
                let ray_pos = first_ray + pixel_x * x as f32 - pixel_y * y as f32;
                // if (x == 0 && y == 0) || (x == self.image_size[0] - 1 && y == 0) || (x == 0 && y == self.image_size[1] - 1) || (x == self.image_size[0] - 1 && y == self.image_size[1] - 1) {
                //     println!("{:?}", ray_pos);
                // }
                ray_data.push((
                    Vector2::new(x as f32, (self.image_size[1] - y) as f32).extend().extend().into(),
                    ray_pos.extend().into(),
                ));
            }
        }


        let mut rays: Vec<raytrace_shader::Ray> = Vec::new();
        for datum in ray_data {
            // println!("{:?}", datum.1);
            rays.push(raytrace_shader::Ray {
                img_pos: datum.0,
                sample_centre: datum.1,
            });
        }

        let num_rays = rays.len() as u32;
        self.ray_data = (create_shader_data_buffer(rays, context, BufferType::Storage), num_rays);
        // self.pixel_dims = [viewport_width / self.image_size[0] as f32, viewport_height / self.image_size[1] as f32];
        self.sample_data = (jitter_size, samples_per_pixel, max_bounces, use_environment_lighting);
    }

    pub fn update_spheres(
        &mut self,
        context: &VulkanoContext,
        sphere_data: Vec<Sphere>
    ) {
        let mut spheres: Vec<raytrace_shader::Sphere> = Vec::new();

        for sphere in sphere_data.iter() {
            spheres.push((*sphere).into());
        }

        let num_spheres = spheres.len() as u32;
        self.sphere_data = (create_shader_data_buffer(spheres, context, BufferType::Storage), num_spheres);
    }

    pub fn update_meshes<T: graphics::Position + BufferContents + Copy + Clone>(
        &mut self,
        context: &VulkanoContext,
        mesh_data: &Vec<RayTracingMesh<T>>
    ) {
        let num_meshes = mesh_data.len() as u32;
        // println!("{:?}", num_meshes);
        let (tri_buffer, mesh_buffer) = transform_meshes(context, mesh_data);
        self.mesh_data = (tri_buffer, mesh_buffer, num_meshes);
    }

    pub fn compute(
        &mut self,
        before_future: Box<dyn GpuFuture>,
        camera: &Camera,
    ) -> Box<dyn GpuFuture> {

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        self.dispatch(&mut builder, camera);


        let command_buffer = builder.build().unwrap();
        let after_future = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        // after_future.wait(None).unwrap();

        after_future.boxed()
    }

    fn dispatch(
        &self,
        builder: &mut AutoCommandBufferBuilder<
        PrimaryAutoCommandBuffer,
        Arc<StandardCommandBufferAllocator>>,
        camera: &Camera,
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
        
        let to_process = (self.ray_data.1 - 1) as u32 / WORKGROUP_SIZE + 1;

        let push_constants = raytrace_shader::PushConstants {
            cam_pos: camera.position.extend().into(),
            cam_alignment_mat: self.get_view_matrix(camera),
            num_rays: self.ray_data.1 as i32,
            num_spheres: self.sphere_data.1 as i32,
            num_meshes: self.mesh_data.2 as i32,
            num_samples: self.sample_data.1 as i32,
            jitter_size: self.sample_data.0,
            max_bounces: self.sample_data.2 as i32,
            use_environment_light: self.sample_data.3 as u32,
        };


        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([to_process, 1, 1])
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


fn transform_meshes<T: graphics::Position + BufferContents + Copy + Clone>(
    context: &VulkanoContext,
    meshes: &Vec<RayTracingMesh<T>>,
) -> (Subbuffer<[raytrace_shader::Triangle]>, Subbuffer<[raytrace_shader::Mesh]>) {
    let mut tri_count = 0;
    let mut tris: Vec<raytrace_shader::Triangle> = Vec::new();
    let mut mesh_data: Vec<raytrace_shader::Mesh> = Vec::new();
    for mesh in meshes.iter() {

        let mat = mesh.material;
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
            material: mat.into(),
            min_point: [min_x, min_y, min_z],
            max_point: [max_x, max_y, max_z]
        });
        tri_count += num_tris;

    }

    let tri_buffer = create_shader_data_buffer(tris, context, BufferType::Storage);
    let mesh_buffer = create_shader_data_buffer(mesh_data, context, BufferType::Storage);
    (tri_buffer, mesh_buffer)
}