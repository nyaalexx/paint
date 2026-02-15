use std::ops::RangeInclusive;

use bevy_reflect::{Reflect, TypeInfo, Typed};
use glam::{UVec2, Vec2};
use serde::{Deserialize, Serialize};

use super::{Context, Texture};
use crate::color::LinearSrgba;

pub trait BrushEngine {
    type Stroke: BrushStroke;

    fn settings() -> BrushSettings;

    fn begin_stroke(&self, settings: &StrokeSettings) -> Self::Stroke;
}

pub trait BrushStroke {
    type Texture: Texture;
    type Context: Context;

    fn update(&mut self, state: &BrushState);

    fn get_texture(&mut self, ctx: &mut Self::Context) -> Self::Texture;
}

#[derive(Debug)]
pub struct StrokeSettings {
    pub canvas_resolution: UVec2,
    pub brush_settings: Box<dyn Reflect>,
}

#[derive(Debug, Clone, Copy)]
pub struct BrushState {
    pub position: Vec2,
    pub pressure: f32,
    // TODO: tilt, orientation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushSettings {
    pub settings: Vec<BrushSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushSetting {
    pub name: String,
    pub kind: BrushSettingKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BrushSettingKind {
    LinearSrgbaColor,
    F32 { min: f32, max: f32 },
}

impl BrushSettings {
    pub fn new<T: Typed>() -> Self {
        BrushSettings::from_type_info(T::type_info())
    }

    pub fn from_type_info(info: &TypeInfo) -> Self {
        let info = info.as_struct().expect("brush settings must be a struct");
        let mut settings = vec![];

        for field in info.iter() {
            let name = field.name().to_string();

            let kind = if field.ty().is::<f32>() {
                let range = field.get_attribute::<RangeInclusive<f32>>();
                let min = range.as_ref().map(|r| *r.start()).unwrap_or(0.0);
                let max = range.as_ref().map(|r| *r.end()).unwrap_or(1.0);

                BrushSettingKind::F32 { min, max }
            } else if field.ty().is::<LinearSrgba>() {
                BrushSettingKind::LinearSrgbaColor
            } else {
                panic!("unknown type used in brush settings")
            };

            settings.push(BrushSetting { name, kind });
        }

        Self { settings }
    }
}
