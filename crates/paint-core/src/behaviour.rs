use glam::{Affine2, UVec2, Vec2};

pub trait Texture: std::fmt::Debug + Send + Sync + Clone {}

pub trait FrameContext {}

#[derive(Debug, Clone)]
pub enum Event {
    SetCanvasResolution(UVec2),
    SetViewportTransform(Affine2),
    BeginBrushStroke,
    UpdateBrushStroke(BrushState),
    EndBrushStroke,
}

#[derive(Debug, Clone)]
pub struct StrokeSettings {
    pub canvas_resolution: UVec2,
}

#[derive(Debug, Clone, Copy)]
pub struct BrushState {
    pub position: Vec2,
    pub pressure: f32,
    // TODO: tilt, orientation
}

pub trait BrushEngine {
    type Stroke: BrushStroke;

    fn begin_stroke(&self, settings: &StrokeSettings) -> Self::Stroke;
}

pub trait BrushStroke {
    type Texture: Texture;
    type FrameContext: FrameContext;

    fn update(&mut self, state: &BrushState);

    fn render(&mut self, ctx: &mut Self::FrameContext) -> Self::Texture;
}

pub trait Compositor {
    type Texture: Texture;
    type FrameContext: FrameContext;

    fn put_texture(&mut self, texture: Self::Texture);

    fn render(&mut self, ctx: &mut Self::FrameContext) -> Self::Texture;
}
