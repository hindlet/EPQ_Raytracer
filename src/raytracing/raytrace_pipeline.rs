use super::*;

mod raytrace_shader {
    graphics::shader!{
        ty: "compute",
        path: "assets/raytracing.glsl",
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
    cam_pos: Vector3
}


impl RayTracePipeine {
    pub fn new(
        context: &VulkanoContext,
        command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>,
        image_size: [u32; 2]
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
            pos: [0.0, 0.0, 0.0, 0.0],
            dir: [0.0, 0.0, 0.0, 0.0]
        };
        let null_sphere = raytrace_shader::Sphere {
            centre: [0.0, 0.0, 0.0],
            radius: 0.0
        };
        

        RayTracePipeine {
            compute_queue: context.graphics_queue().clone(),
            compute_pipeline: pipeline,
            command_buffer_allocator: command_buffer_allocator.clone(),
            descriptor_set_allocator: descriptor_set_allocator.clone(),
            image: image,
            image_size: image_size,

            ray_data: (create_shader_data_buffer(vec![null_ray], context, BufferType::Storage), 0),
            sphere_data: (create_shader_data_buffer(vec![null_sphere], context, BufferType::Storage), 0),
            cam_pos: Vector3::ZERO
        }
    }

    fn get_pipeline_layout(
        context: &VulkanoContext
    ) -> Arc<PipelineLayout> {

        let mut bindings = BTreeMap::new();

        bindings.insert(0, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageImage));
        bindings.insert(1, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer));
        bindings.insert(2, DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer));

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


        PipelineLayout::new(context.device().clone(),
            PipelineLayoutCreateInfo {
                set_layouts: vec![set_layout],
                push_constant_ranges: vec![PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    offset: 0,
                    size: (size_of::<i32>() * 2 + size_of::<f32>() * 3) as u32
                }],
                ..Default::default()
            }
        ).unwrap()
    }


    pub fn image(&self) -> DeviceImageView {
        self.image.clone()
    }

    /// this assumes a fixed camera, if not the data would need to be updated
    pub fn init_data(
        &mut self,
        context: &VulkanoContext,
        camera: &Camera,
        camera_focal_length: f32,
        viewport_height: f32,
    ) {
        let viewport_width = viewport_height * (self.image_size[0] as f32 / self.image_size[1] as f32);

        let viewport_x = camera.up.cross(camera.direction).normalised();
        let viewport_y = viewport_x.cross(camera.direction).normalised();
        let viewport_upper_left = camera.position + camera.direction * camera_focal_length - (viewport_x * viewport_width - viewport_y * viewport_height) * 0.5;

        let pixel_x = viewport_x * viewport_width / self.image_size[0] as f32;
        let pixel_y = viewport_y * viewport_height / self.image_size[1] as f32;

        let first_ray = viewport_upper_left + (pixel_x - pixel_y) * 0.5;
        let mut ray_data: Vec<([f32; 4], [f32; 4])> = Vec::new();

        for y in 0..self.image_size[1] {
            for x in 0..self.image_size[0] {
                let ray_pos = first_ray + pixel_x * x as f32 - pixel_y * y as f32;
                ray_data.push((
                    Vector2::new(x as f32, y as f32).extend().extend().into(),
                    (ray_pos - camera.position).normalised().extend().into()
                ));
            }
        }


        let mut rays: Vec<raytrace_shader::Ray> = Vec::new();
        for datum in ray_data {
            // println!("{:?}", datum.1);
            rays.push(raytrace_shader::Ray {
                pos: datum.0,
                dir: datum.1,
            });
        }

        let num_rays = rays.len() as u32;
        self.ray_data = (create_shader_data_buffer(rays, context, BufferType::Storage), num_rays);
        self.cam_pos = camera.position;
    }

    pub fn update_spheres(
        &mut self,
        context: &VulkanoContext,
        sphere_data: Vec<([f32; 3], f32)>
    ) {
        let mut spheres: Vec<raytrace_shader::Sphere> = Vec::new();

        for datum in sphere_data.iter() {
            spheres.push(raytrace_shader::Sphere {
                centre: datum.0,
                radius: datum.1
            });
        }

        let num_spheres = spheres.len() as u32;
        self.sphere_data = (create_shader_data_buffer(spheres, context, BufferType::Storage), num_spheres);
    }

    pub fn compute(
        &mut self,
        before_future: Box<dyn GpuFuture>
    ) -> Box<dyn GpuFuture> {

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        self.dispatch(&mut builder);


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
    ) {
        let pipeline_layout = self.compute_pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.image.clone()),
                WriteDescriptorSet::buffer(1, self.ray_data.0.clone()),
                WriteDescriptorSet::buffer(2, self.sphere_data.0.clone())
            ],
        )
        .unwrap();
        
        let to_process = ((self.ray_data.1 - 1)as u32 / 64) * 64 + 64;

        let push_constants = raytrace_shader::PushConstants {
            camera_pos: self.cam_pos.into(),
            num_rays: self.ray_data.1 as i32,
            num_spheres: self.sphere_data.1 as i32,
        };


        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([to_process / 64, 1, 1])
            .unwrap();
    }
}