use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::bind_group_layouts;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Key {
    pub bind_group_layouts: Vec<bind_group_layouts::Key>,
    pub immediate_size: u32,
}

impl Key {
    pub fn create(
        &self,
        device: &wgpu::Device,
        bind_group_layouts: &bind_group_layouts::Storage,
    ) -> wgpu::PipelineLayout {
        let bgs = self
            .bind_group_layouts
            .iter()
            .cloned()
            .map(|key| bind_group_layouts.get(key))
            .collect::<Vec<_>>();

        let bg_refs = bgs.iter().collect::<Vec<_>>();

        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bg_refs,
            immediate_size: self.immediate_size,
        })
    }
}

#[derive(Debug)]
pub struct Storage {
    device: wgpu::Device,
    bind_group_layouts: Arc<bind_group_layouts::Storage>,
    pipeline_layouts: Mutex<HashMap<Key, wgpu::PipelineLayout>>,
}

impl Storage {
    pub fn new(device: wgpu::Device, bind_group_layouts: Arc<bind_group_layouts::Storage>) -> Self {
        Self {
            device,
            bind_group_layouts,
            pipeline_layouts: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: Key) -> wgpu::PipelineLayout {
        let mut pipeline_layouts = self.pipeline_layouts.lock().unwrap();
        pipeline_layouts
            .entry(key)
            .or_insert_with_key(|key| key.create(&self.device, &self.bind_group_layouts))
            .clone()
    }
}
