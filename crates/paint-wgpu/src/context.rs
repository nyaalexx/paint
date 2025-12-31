use std::sync::Arc;

use crate::{bind_group_layouts, pipeline_layouts, render_pipelines, shaders};

#[derive(Debug)]
pub struct Context {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) bind_group_layouts: Arc<bind_group_layouts::Storage>,
    pub(crate) render_pipelines: Arc<render_pipelines::Storage>,
    pub(crate) default_texture_view: wgpu::TextureView,
    pub(crate) default_sampler: wgpu::Sampler,
}

impl Context {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let shaders = Arc::new(shaders::Storage::new(device.clone()));
        let bind_group_layouts = Arc::new(bind_group_layouts::Storage::new(device.clone()));

        let pipeline_layouts = Arc::new(pipeline_layouts::Storage::new(
            device.clone(),
            bind_group_layouts.clone(),
        ));

        let render_pipelines = Arc::new(render_pipelines::Storage::new(
            device.clone(),
            shaders,
            pipeline_layouts,
        ));

        let default_texture = crate::utils::create_default_texture(&device, &queue);
        let default_texture_view = default_texture.create_view(&Default::default());
        let default_sampler = crate::utils::create_default_sampler(&device);

        Self {
            device,
            queue,
            bind_group_layouts,
            render_pipelines,
            default_texture_view,
            default_sampler,
        }
    }
}

#[derive(Debug)]
pub struct FrameContext {
    pub(crate) encoder: wgpu::CommandEncoder,
}

impl FrameContext {
    pub fn new(ctx: &Context) -> Self {
        Self {
            encoder: ctx.device.create_command_encoder(&Default::default()),
        }
    }
}

impl paint_core::behaviour::FrameContext for FrameContext {}
