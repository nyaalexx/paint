pub mod single_sampled_texture;

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Key {
    SingleSampledTexture,
}

impl Key {
    pub fn create(self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        match self {
            Key::SingleSampledTexture => self::single_sampled_texture::create(device),
        }
    }
}

#[derive(Debug)]
pub struct Storage {
    device: wgpu::Device,
    bind_group_layouts: Mutex<HashMap<Key, wgpu::BindGroupLayout>>,
}

impl Storage {
    pub fn new(device: wgpu::Device) -> Self {
        Self {
            device,
            bind_group_layouts: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: Key) -> wgpu::BindGroupLayout {
        let mut bind_group_layouts = self.bind_group_layouts.lock().unwrap();
        bind_group_layouts
            .entry(key)
            .or_insert_with(|| key.create(&self.device))
            .clone()
    }
}
