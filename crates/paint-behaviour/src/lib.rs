use glam::{Affine2, UVec2};
use paint_core::behaviour::{
    BrushEngine, BrushStroke, Compositor, Event, FrameContext, StrokeSettings, Texture,
};
use paint_core::presentation;

pub trait Impls {
    type Texture: Texture;
    type FrameContext: FrameContext;
    type Compositor: Compositor<Texture = Self::Texture, FrameContext = Self::FrameContext>;
    type BrushEngine: BrushEngine<Stroke = Self::BrushStroke>;
    type BrushStroke: BrushStroke<Texture = Self::Texture, FrameContext = Self::FrameContext>;
}

pub struct Behaviour<I: Impls> {
    state: State<I>,
    compositor: I::Compositor,
    brush_engine: I::BrushEngine,
}

struct State<I: Impls> {
    canvas_resolution: UVec2,
    viewport_transform: Affine2,
    brush_stroke: Option<I::BrushStroke>,
}

impl<I: Impls> Behaviour<I> {
    pub fn new(compositor: I::Compositor, brush_engine: I::BrushEngine) -> Self {
        Self {
            state: State {
                canvas_resolution: UVec2::new(2304, 1440),
                viewport_transform: Affine2::IDENTITY,
                brush_stroke: None,
            },
            compositor,
            brush_engine,
        }
    }

    pub fn handle_event(&mut self, frame_ctx: &mut I::FrameContext, event: Event) {
        match event {
            Event::SetCanvasResolution(resolution) => {
                self.state.canvas_resolution = resolution;
            }

            Event::SetViewportTransform(transform) => {
                self.state.viewport_transform = transform;
            }

            Event::BeginBrushStroke => {
                self.state.brush_stroke = Some(self.brush_engine.begin_stroke(&StrokeSettings {
                    canvas_resolution: self.state.canvas_resolution,
                }));
            }

            Event::UpdateBrushStroke(state) => {
                if let Some(stroke) = &mut self.state.brush_stroke {
                    stroke.update(&state);
                }
            }

            Event::EndBrushStroke => {
                if let Some(mut stroke) = self.state.brush_stroke.take() {
                    let stroke_texture = stroke.render(frame_ctx);
                    self.compositor.put_texture(stroke_texture);
                }
            }
        }
    }

    pub fn present(
        &mut self,
        frame_ctx: &mut I::FrameContext,
    ) -> presentation::Viewport<I::Texture> {
        let mut layers = Vec::new();

        let composite = self.compositor.render(frame_ctx);
        layers.push(presentation::Layer::Texture(composite));

        if let Some(stroke) = &mut self.state.brush_stroke {
            let texture = stroke.render(frame_ctx);
            layers.push(presentation::Layer::Texture(texture));
        }

        presentation::Viewport {
            transform: self.state.viewport_transform,
            canvas: presentation::Canvas {
                resolution: self.state.canvas_resolution,
                layers,
            },
        }
    }
}
