use glam::{Affine2, UVec2, Vec2};

use crate::{persistence, presentation};

/// App behaviour implementation.
///
/// Handles input events, performs high level orchestration and generates output
/// actions, consumed by the presentation layer.
///
/// The behaviour runs in a separate thread, processess input events in batches,
/// then performs a sequence of actions. Usually this is cycle is tied to the
/// screen's refresh rate.
///
/// Context argument is meant for reusing state during a single frame, such as
/// command encoders.
pub trait Behaviour {
    /// Lower-level impls.
    type Impls: Impls;

    /// Handle an input event.
    fn handle_event(&mut self, ctx: &mut <Self::Impls as Impls>::Context, event: Event);

    /// Perform an action.
    fn perform_action(
        &mut self,
        ctx: &mut <Self::Impls as Impls>::Context,
    ) -> Option<Action<Self::Impls>>;
}

/// Collection of traits designed to work together, which implement actual
/// lower level logic.
pub trait Impls {
    type Texture: Texture;
    type Context: Context;
    type Compositor: Compositor<Texture = Self::Texture, Context = Self::Context>;
    type BrushEngine: BrushEngine<Stroke = Self::BrushStroke>;
    type BrushStroke: BrushStroke<Texture = Self::Texture, Context = Self::Context>;
}

pub trait Texture: std::fmt::Debug + Send + Sync + Clone + 'static {}

pub trait DownloadedTexture: std::fmt::Debug + Send + Sync + 'static {
    fn as_persistence(&self) -> persistence::Texture<'_>;
}

pub trait Context {}

/// An input event.
#[derive(Debug, Clone)]
pub enum Event {
    InvalidateViewport,
    SetCanvasResolution(UVec2),
    SetViewportTransform(Affine2),
    BeginBrushStroke,
    UpdateBrushStroke(BrushState),
    EndBrushStroke,
}

/// A presentation action.
///
/// This actions affect what the user can see on the screen.
#[derive(Debug, Clone)]
pub enum Action<I: Impls> {
    PresentViewport(presentation::Viewport<I::Texture>),
}

pub trait BrushEngine {
    type Stroke: BrushStroke;

    fn begin_stroke(&self, settings: &StrokeSettings) -> Self::Stroke;
}

pub trait BrushStroke {
    type Texture: Texture;
    type Context: Context;

    fn update(&mut self, state: &BrushState);

    fn render(&mut self, ctx: &mut Self::Context) -> Self::Texture;
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

pub trait Compositor {
    type Texture: Texture;
    type Context: Context;

    fn put_texture(&mut self, ctx: &mut Self::Context, texture: Self::Texture);

    fn render(&mut self, ctx: &mut Self::Context) -> Self::Texture;

    fn download(
        &mut self,
        ctx: &mut Self::Context,
    ) -> impl Future<Output = impl DownloadedTexture> + Send + 'static;
}
