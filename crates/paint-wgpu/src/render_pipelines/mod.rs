pub mod canvas_border;
pub mod fullscreen_triangle;
pub mod fullscreen_triangle_interpolate_two_textures;
pub mod single_quad;
pub mod stamped_brush;

// TODO: add pipeline cache

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{pipeline_layouts, shaders};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Key {
    FullscreenTriangle,
    FullscreenTriangleInterpolateTwoTextures,
    SingleQuad,
    StampedBrush,
    CanvasBorder,
}

impl Key {
    pub fn compile(
        self,
        device: &wgpu::Device,
        shaders: &shaders::Storage,
        pipeline_layouts: &pipeline_layouts::Storage,
    ) -> wgpu::RenderPipeline {
        match self {
            Key::FullscreenTriangle => {
                self::fullscreen_triangle::compile(device, shaders, pipeline_layouts)
            }
            Key::FullscreenTriangleInterpolateTwoTextures => {
                self::fullscreen_triangle_interpolate_two_textures::compile(
                    device,
                    shaders,
                    pipeline_layouts,
                )
            }
            Key::SingleQuad => self::single_quad::compile(device, shaders, pipeline_layouts),
            Key::StampedBrush => self::stamped_brush::compile(device, shaders, pipeline_layouts),
            Key::CanvasBorder => self::canvas_border::compile(device, shaders, pipeline_layouts),
        }
    }
}

#[derive(Debug)]
pub struct Storage {
    device: wgpu::Device,
    shaders: Arc<shaders::Storage>,
    pipeline_layouts: Arc<pipeline_layouts::Storage>,
    pipelines: Mutex<HashMap<Key, wgpu::RenderPipeline>>,
}

impl Storage {
    pub fn new(
        device: wgpu::Device,
        shaders: Arc<shaders::Storage>,
        pipeline_layouts: Arc<pipeline_layouts::Storage>,
    ) -> Self {
        Self {
            device,
            shaders,
            pipeline_layouts,
            pipelines: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: Key) -> wgpu::RenderPipeline {
        let mut pipelines = self.pipelines.lock().unwrap();
        pipelines
            .entry(key)
            .or_insert_with(|| key.compile(&self.device, &self.shaders, &self.pipeline_layouts))
            .clone()
    }
}
