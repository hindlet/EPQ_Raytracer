use std::sync::Arc;
use graphics::*;
use graphics::all_vulkano::{
    format::Format,
    buffer::BufferContents
};
use graphics::all_vulkano_utils::{window::{VulkanoWindows, WindowDescriptor}, context::VulkanoConfig};
use super::{
    diffuse::DiffusePipeline,
    raytrace_pipeline::{RayTracePipeline, RayTracerSettings},
    texture_draw_pipeline::RenderPassOverFrame,
};


pub struct RayTracingApp<T: graphics::Position + BufferContents + Copy + Clone> {
    pub context: VulkanoContext,
    pub windows: VulkanoWindows,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    pub pipeline: Option<(RayTracePipeline, DiffusePipeline, RenderPassOverFrame)>,
    frame: u32,
    pub camera: Camera,
    settings: RayTracerSettings<T>
}


impl<T: graphics::Position + BufferContents + Copy + Clone> RayTracingApp<T> {
    /// create a new raytracing app
    pub fn new(
        camera: Camera,
        settings: RayTracerSettings<T>
    ) -> Self {

        let context = VulkanoContext::new(VulkanoConfig::default());
        let command_allocator = Arc::new(StandardCommandBufferAllocator::new(
            context.device().clone(),
            Default::default()
        ));
        let descript_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            context.device().clone()
        ));
        
        RayTracingApp {
            context,
            command_buffer_allocator: command_allocator,
            descriptor_set_allocator: descript_allocator,
            windows: VulkanoWindows::default(),
            pipeline: None,
            frame: 0,
            camera,
            settings
        }
    }


    /// intitialise all pipelines and open window
    pub fn open(
        &mut self,
        event_loop: &EventLoop<()>,
        image_size: [u32; 2]
    ) {
        self.windows.create_window(
            event_loop,
            &self.context,
            &WindowDescriptor {
                width: image_size[0] as f32,
                height: image_size[1] as f32,
                title: "Raytracing".to_string(),
                ..Default::default()
            },
            |_| {}
        );

        let raytrace_pipeline = RayTracePipeline::new(
            &self.context,
            &self.command_buffer_allocator,
            &self.descriptor_set_allocator,
            image_size,
            self.settings.clone()
        );
        let mut diffuse_pipeline = DiffusePipeline::new(
            &self.context,
            image_size,
            &self.command_buffer_allocator,
            &self.descriptor_set_allocator
        );
        let render_pass = RenderPassOverFrame::new(
            &self.context,
            &self.command_buffer_allocator,
            &self.descriptor_set_allocator,
            Format::B8G8R8A8_UNORM
        );

        let window_renderer = self.windows.get_primary_renderer_mut().unwrap();
        match window_renderer.window_size() {
            [w, h] => {
                if w == 0.0 || h == 0.0 {
                    return;
                }
            }
        }

        let before_init_future = match window_renderer.acquire() {
            Err(e) => {
                println!("{e}");
                return;
            }
            Ok(future) => future
        };

        let after_raytrace_init_future = raytrace_pipeline.init(before_init_future);
        let after_diffuse_future = diffuse_pipeline.next_frame(self.frame, raytrace_pipeline.image(), after_raytrace_init_future);

        let image = diffuse_pipeline.image();
        let target_image = window_renderer.swapchain_image_view();

        let after_render = render_pass.render(after_diffuse_future, image, target_image);

        window_renderer.present(after_render, true);

        self.pipeline = Some((raytrace_pipeline, diffuse_pipeline, render_pass));
        self.frame += 1;
    }

    
}


/// handle input like window closing and camera control
pub fn handle_events<T: graphics::Position + BufferContents + Copy + Clone>(
    app: &mut RayTracingApp<T>,
    event_loop: &mut EventLoop<()>
) -> bool{
    let id = app.windows.primary_window_id().unwrap();
    generic_winit_event_handling_with_camera(event_loop, &mut app.windows, &mut Vec::new(), (&mut app.camera, &id))
}

/// computes the next frame and render
pub fn compute_then_render<T: graphics::Position + BufferContents + Copy + Clone>(
    app: &mut RayTracingApp<T>,
    frame_time: f32,
) {
    let window_renderer = app.windows.get_primary_renderer_mut().unwrap();
    match window_renderer.window_size() {
        [w, h] => {
            if w == 0.0 || h == 0.0 {
                return;
            }
        }
    }

    app.camera.do_move(frame_time);

    let (raytrace_pipeline, diffuse_pipeline, render_pipeline) = app.pipeline.as_mut().unwrap();

    let before_pipeline_future = match window_renderer.acquire() {
        Err(e) => {
            println!("{e}");
            return;
        }
        Ok(future) => future,
    };

    let after_raytrace = raytrace_pipeline.compute(before_pipeline_future, &app.camera, app.frame);
    let raytrace_image = raytrace_pipeline.image();

    let after_diffuse = diffuse_pipeline.next_frame(app.frame, raytrace_image, after_raytrace);
    let diffuse_image = diffuse_pipeline.image();

    let target_image = window_renderer.swapchain_image_view();

    let after_render = render_pipeline
        .render(after_diffuse, diffuse_image, target_image);

    window_renderer.present(after_render, true);
    app.frame += 1;
}

pub fn compute_n_then_render<T: graphics::Position + BufferContents + Copy + Clone>(
    app: &mut RayTracingApp<T>,
    num_renders: usize
) {
    let window_renderer = app.windows.get_primary_renderer_mut().unwrap();
    let (raytrace_pipeline, diffuse_pipeline, render_pipeline) = app.pipeline.as_mut().unwrap();


    let mut last_future = match window_renderer.acquire() {
        Err(e) => {
            println!("{e}");
            return;
        }
        Ok(future) => future,
    };
    for _ in 0..num_renders {
        let after_raytrace = raytrace_pipeline.compute(last_future, &app.camera, app.frame);
        let raytrace_image = raytrace_pipeline.image();

        last_future = diffuse_pipeline.next_frame(app.frame, raytrace_image, after_raytrace);
        app.frame += 1;
    }

    let diffuse_image = diffuse_pipeline.image();

    let target_image = window_renderer.swapchain_image_view();

    let after_render = render_pipeline
        .render(last_future, diffuse_image, target_image);

    window_renderer.present(after_render, true);
}