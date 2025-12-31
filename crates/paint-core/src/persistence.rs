use std::collections::HashMap;

use chrono::{DateTime, Utc};
use glam::UVec2;

#[derive(Debug, Clone)]
pub struct Project {
    pub creation_time: DateTime<Utc>,
    pub modify_time: DateTime<Utc>,
    pub resolution: UVec2,
    pub ui_state: UiState,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub map: HashMap<String, Vec<u8>>,
}
