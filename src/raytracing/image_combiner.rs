use super::*;


mod image_combine_shader {
    graphics::shader!{
        ty: "compute",
        path: "assets/image_combiner.glsl"
    }
}


pub struct ImageCombiner {
    image: DeviceImageView,
    image_count: u32,
    image_size: [u32; 2],

    compute_queue: Arc<Queue>,
    compute_pipeline: Arc<ComputePipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}


impl ImageCombiner {

    pub fn new(
        context: &VulkanoContext,
        image_size: [u32; 2],
        command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>,
    ) -> Self {

        let pipeline = ComputePipeline::new(
            context.device().clone(),
            image_combine_shader::load(context.device().clone()).unwrap().entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        ).unwrap();

        let image = StorageImage::general_purpose_image_view(
            context.memory_allocator(),
            context.compute_queue().clone(),
            image_size,
            Format::R8G8B8A8_UNORM,
            ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
        ).unwrap();

        ImageCombiner {
            image: image,
            image_count: 0,
            compute_queue: context.graphics_queue().clone(),
            compute_pipeline: pipeline,
            image_size,
            command_buffer_allocator: command_buffer_allocator.clone(),
            descriptor_set_allocator: descriptor_set_allocator.clone()
        }
    }


    pub fn image(&self) -> DeviceImageView {
        self.image.clone()
    }

    pub fn next_frame(
        &mut self,
        next_image: DeviceImageView,
        before_future: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        let group_number = (self.image_size[0] * self.image_size[1] - 1) / 256 + 1;

        self.dispatch(&mut builder, next_image, self.image_count, group_number);
        self.image_count += 1;

        let command_buffer = builder.build().unwrap();
        let after_future = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        after_future.boxed()
    }

    fn dispatch(
        &self,
        builder: &mut AutoCommandBufferBuilder<
        PrimaryAutoCommandBuffer,
        Arc<StandardCommandBufferAllocator>>,
        image: DeviceImageView,
        frame_num: u32,
        group_number: u32
    ) {

        let pipeline_layout = self.compute_pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            desc_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.image.clone()),
                WriteDescriptorSet::image_view(1, image)
            ]
        ).unwrap();

        let push_constants = image_combine_shader::PushConstants {
            frame: frame_num,
            image_width: self.image_size[0],
            image_height: self.image_size[1]
        };

        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([group_number, 1, 1])
            .unwrap();
    }


}