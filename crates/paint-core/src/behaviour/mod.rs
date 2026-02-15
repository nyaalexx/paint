mod brush;

use std::path::PathBuf;

use glam::{Affine2, UVec2};

pub use self::brush::*;
use crate::persistence::{self, Project};
use crate::presentation;

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
pub trait Impls: 'static {
    type Context: Context;
    type Texture: Texture<Context = Self::Context>;
    type Compositor: Compositor<Context = Self::Context, Texture = Self::Texture>;
    type BrushEngine: BrushEngine<Stroke = Self::BrushStroke>;
    type BrushStroke: BrushStroke<Context = Self::Context, Texture = Self::Texture>;
    type Project: Project;
}

pub trait Context {}

pub trait Texture: std::fmt::Debug + Send + Sync + Clone + 'static {
    type Context: Context;
    type Downloaded: DownloadedTexture;

    fn upload(ctx: &mut Self::Context, texture: persistence::Texture<'_>) -> Self;

    fn download(
        &self,
        ctx: &mut Self::Context,
    ) -> impl Future<Output = Self::Downloaded> + Send + 'static;
}

pub trait DownloadedTexture: std::fmt::Debug + Send + Sync + 'static {
    fn as_persistence(&self) -> persistence::Texture<'_>;
}

/// An input event.
#[derive(Debug, Clone)]
pub enum Event {
    InvalidateViewport,
    SetCanvasResolution(UVec2),
    SetViewportTransform(Affine2),
    BeginBrushStroke,
    UpdateBrushStroke(BrushState),
    EndBrushStroke,
    Save(PathBuf),
    Open(PathBuf),
}

/// A presentation action.
///
/// This actions affect what the user can see on the screen.
#[derive(Debug, Clone)]
pub enum Action<I: Impls> {
    PresentViewport(presentation::Viewport<I::Texture>),
}

pub trait Compositor {
    type Texture: Texture;
    type Context: Context;

    fn put_texture(&mut self, ctx: &mut Self::Context, texture: Self::Texture);

    fn get_composite(&mut self, ctx: &mut Self::Context) -> Self::Texture;
}
