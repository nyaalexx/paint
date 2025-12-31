// TODO: add spir-v cache

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Key {
    SingleQuad,
    StampedBrush,
}

impl Key {
    pub fn compile(self, device: &wgpu::Device) -> wgpu::ShaderModule {
        let wgsl = match self {
            Key::SingleQuad => include_str!("wgsl/single_quad.wgsl"),
            Key::StampedBrush => include_str!("wgsl/stamped_brush.wgsl"),
        };

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{self:?} Shader")),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        })
    }
}

#[derive(Debug)]
pub struct Storage {
    device: wgpu::Device,
    shaders: Mutex<HashMap<Key, wgpu::ShaderModule>>,
}

impl Storage {
    pub fn new(device: wgpu::Device) -> Self {
        Self {
            device,
            shaders: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: Key) -> wgpu::ShaderModule {
        let mut shaders = self.shaders.lock().unwrap();
        shaders
            .entry(key)
            .or_insert_with(|| key.compile(&self.device))
            .clone()
    }
}
