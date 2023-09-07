use std::sync::Arc;
use graphics::*;
use graphics::all_vulkano::{
    memory::allocator::StandardMemoryAllocator,
    device::Queue,
    render_pass::{RenderPass, Subpass, Framebuffer, FramebufferCreateInfo},
    pipeline::{Pipeline, GraphicsPipeline, graphics::{viewport::{Viewport, ViewportState}, vertex_input::VertexBufferDescription, input_assembly::InputAssemblyState, multisample::MultisampleState, depth_stencil::DepthStencilState}, PipelineBindPoint},
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageViewAbstract, SampleCount},
    command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents},
    sync::GpuFuture,
    buffer::Subbuffer,
    single_pass_renderpass,
    format::Format,
    shader::ShaderModule,
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator},
    padded::Padded
};
use graphics::all_vulkano_utils::renderer::SwapchainImageView;
use maths::{Matrix4, Vector3};

mod vs {
    graphics::shader!{
        ty: "vertex",
        path: "assets/phong_vert.glsl"
    }
}

mod fs {
    graphics::shader!{
        ty: "fragment",
        path: "assets/phong_frag.glsl"
    }
}

impl Default for fs::Light {
    fn default() -> Self {
        fs::Light {
            position: [0.0, 0.0, 0.0],
            intensity: 0.0,
            colour: [0.0, 0.0, 0.0]
        }
    }
}


pub struct Light {
    pub pos: Vector3,
    pub strength: f32,
    pub colour: Vector3,
}

pub struct PhongPipeline {
    allocator: Arc<StandardMemoryAllocator>,
    queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    intermediary: Arc<ImageView<AttachmentImage>>,
    depth: Arc<ImageView<AttachmentImage>>,
    sample_count: SampleCount,
}

impl PhongPipeline {
    pub fn new(
        context: &VulkanoContext,
        command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
        descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>,
        sample_count: Option<SampleCount>,
    ) -> Self {

        let samples = sample_count.unwrap_or(SampleCount::Sample2);

        let render_pass = Self::create_render_pass(context, samples);
        let pipeline = Self::create_pipeline(&vs::load(context.device().clone()).unwrap(), &fs::load(context.device().clone()).unwrap(), &vertex_defs::coloured_normal(), &render_pass, context, samples);

        let intermediary_image = ImageView::new_default(
            AttachmentImage::transient_multisampled(context.memory_allocator(), [1, 1], samples, Format::B8G8R8A8_SRGB).unwrap()
        ).unwrap();
        let depth = ImageView::new_default(
            AttachmentImage::transient_multisampled(context.memory_allocator(), [1, 1], samples, Format::D16_UNORM).unwrap()
        ).unwrap();
        
        Self {
            allocator: context.memory_allocator().clone(),
            queue: context.graphics_queue().clone(),
            render_pass,
            pipeline,
            command_buffer_allocator: command_buffer_allocator.clone(),
            descriptor_set_allocator: descriptor_set_allocator.clone(),
            intermediary: intermediary_image,
            depth: depth,
            sample_count: samples
        }
    }

    fn create_render_pass(
        context: &VulkanoContext,
        sample_num: SampleCount,
    ) -> Arc<RenderPass> {
        single_pass_renderpass!(
            context.device().clone(),
            attachments: {
                intermediary: {
                    load: Clear,
                    store: DontCare,
                    format: Format::B8G8R8A8_SRGB,
                    samples: sample_num,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: sample_num,
                },
                end: {
                    load: DontCare,
                    store: Store,
                    format: Format::B8G8R8A8_SRGB,
                    samples: 1,
                }
            },
            pass: {
                color: [intermediary],
                depth_stencil: {depth},
                resolve: [end],
            }
        )
        .unwrap()
    }

    fn create_pipeline(
        vertex_shader: &Arc<ShaderModule>,
        fragment_shader: &Arc<ShaderModule>,
        vertex_def: &[VertexBufferDescription],
        render_pass: &Arc<RenderPass>,
        context: &VulkanoContext,
        sample_count: SampleCount
    ) -> Arc<GraphicsPipeline> {

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        GraphicsPipeline::start()
            .vertex_input_state(vertex_def)
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .render_pass(subpass.clone())
            .multisample_state(MultisampleState {
                rasterization_samples: sample_count,
                ..Default::default()
            })
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .build(context.device().clone())
            .unwrap()
    }


    pub fn draw(
        &mut self,
        before_future: Box<dyn GpuFuture>,
        image: SwapchainImageView,

        vertex_buffer: &Subbuffer<[ColouredVertex]>,
        normal_buffer: &Subbuffer<[Normal]>,
        index_buffer: &Subbuffer<[u32]>,
        uniforms: &(Subbuffer<vs::Data>, Subbuffer<fs::Data>),
        lights: &Subbuffer<fs::LightData>
    ) -> Box<dyn GpuFuture>{

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let dimensions = image.image().dimensions().width_height();
        // Resize intermediary image
        if dimensions != self.intermediary.dimensions().width_height() {
            self.intermediary = ImageView::new_default(
                AttachmentImage::transient_multisampled(
                    &self.allocator,
                    dimensions,
                    self.sample_count,
                    image.image().format(),
                )
                .unwrap(),
            )
            .unwrap();
        }
        // Resize depth image
        if dimensions != self.depth.dimensions().width_height() {
            self.depth = ImageView::new_default(
                AttachmentImage::transient_multisampled(
                    &self.allocator,
                    dimensions,
                    self.sample_count,
                    Format::D16_UNORM,
                )
                .unwrap(),
            )
            .unwrap();
        }

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniforms.0.clone()), WriteDescriptorSet::buffer(1, uniforms.1.clone()), WriteDescriptorSet::buffer(2, lights.clone())],
        )
        .unwrap();

        let framebuffer = Framebuffer::new(self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![self.intermediary.clone(), self.depth.clone(), image],
            ..Default::default()
        })
        .unwrap();

        // Begin render pipeline commands
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some(1f32.into()),
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::Inline,
            )
            .unwrap();


        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                set,
            )
            .set_viewport(0, vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }])
            .bind_vertex_buffers(0, (vertex_buffer.clone(), normal_buffer.clone()))
            .bind_index_buffer(index_buffer.clone())
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap();


        builder.end_render_pass().unwrap();
        let command_buffer = builder.build().unwrap();
        let after_future = before_future.then_execute(self.queue.clone(), command_buffer).unwrap();

        after_future.boxed()
    }
}

pub fn get_uniforms(
    swapchain_size: [u32; 2],
    allocator: &SubbufferAllocator,
    camera: &Camera
) -> (Subbuffer<vs::Data>, Subbuffer<fs::Data>) {
    let (view, proj) = get_generic_uniforms(swapchain_size, camera);

    let vertex_uniform_data = vs::Data {
        world: Matrix4::IDENTITY.into(),
        view: view.into(),
        proj: proj.into(),
    };
    let fragment_uniform_data = fs::Data {
        viewpos: camera.position.into(),
    };

    let vert_buffer = allocator.allocate_sized().unwrap();
    *vert_buffer.write().unwrap() = vertex_uniform_data;
    let frag_buffer = allocator.allocate_sized().unwrap();
    *frag_buffer.write().unwrap() = fragment_uniform_data;
    (vert_buffer, frag_buffer)
}

pub fn get_light_buffer(
    allocator: &SubbufferAllocator,
    lights: Vec<Light>
) -> Subbuffer<fs::LightData>{
    let mut transformed_data: [Padded<fs::Light, 4>; 4] = [Padded(fs::Light::default()); 4];
    for i in 0..4 {
        if i >= lights.len() {break;}
        transformed_data[i] = Padded(fs::Light {
            position: lights[i].pos.into(),
            intensity: lights[i].strength,
            colour: lights[i].colour.into()
        })
    }
    let data = fs::LightData {
        lights: transformed_data
    };

    let light_buffer = allocator.allocate_sized().unwrap();
    *light_buffer.write().unwrap() = data;
    light_buffer
}