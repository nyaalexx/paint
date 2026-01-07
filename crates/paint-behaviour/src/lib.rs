use glam::{Affine2, UVec2};
use paint_core::behaviour::{
    Action, BrushEngine, BrushStroke, Compositor, Event, Impls, StrokeSettings,
};
use paint_core::presentation;

pub struct Behaviour<I: Impls> {
    state: State<I>,
    compositor: I::Compositor,
    brush_engine: I::BrushEngine,
}

struct State<I: Impls> {
    viewport_dirty: bool,
    color_picker_dirty: bool,
    canvas_resolution: UVec2,
    viewport_transform: Affine2,
    brush_stroke: Option<I::BrushStroke>,
}

impl<I: Impls> Behaviour<I> {
    pub fn new(compositor: I::Compositor, brush_engine: I::BrushEngine) -> Self {
        Self {
            state: State {
                viewport_dirty: true,
                color_picker_dirty: true,
                canvas_resolution: UVec2::new(2304, 1440),
                viewport_transform: Affine2::IDENTITY,
                brush_stroke: None,
            },
            compositor,
            brush_engine,
        }
    }

    pub fn handle_event(&mut self, ctx: &mut I::Context, event: Event) {
        match event {
            Event::InvalidateViewport => {
                self.state.viewport_dirty = true;
            }

            Event::InvalidateColorPicker => {
                self.state.color_picker_dirty = true;
            }

            Event::SetCanvasResolution(resolution) => {
                self.state.canvas_resolution = resolution;
                self.state.viewport_dirty = true;
            }

            Event::SetViewportTransform(transform) => {
                self.state.viewport_transform = transform;
                self.state.viewport_dirty = true;
            }

            Event::BeginBrushStroke => {
                self.state.brush_stroke = Some(self.brush_engine.begin_stroke(&StrokeSettings {
                    canvas_resolution: self.state.canvas_resolution,
                }));
            }

            Event::UpdateBrushStroke(state) => {
                if let Some(stroke) = &mut self.state.brush_stroke {
                    stroke.update(&state);
                    self.state.viewport_dirty = true;
                }
            }

            Event::EndBrushStroke => {
                if let Some(mut stroke) = self.state.brush_stroke.take() {
                    let stroke_texture = stroke.render(ctx);
                    self.compositor.put_texture(stroke_texture);
                    self.state.viewport_dirty = true;
                }
            }
        }
    }

    pub fn perform_action(&mut self, ctx: &mut I::Context) -> Option<Action<I>> {
        if self.state.viewport_dirty {
            let viewport = self.present_viewport(ctx);
            self.state.viewport_dirty = false;
            return Some(Action::PresentViewport(viewport));
        }

        if self.state.color_picker_dirty {
            let color_picker = self.present_color_picker();
            self.state.color_picker_dirty = false;
            return Some(Action::PresentColorPicker(color_picker));
        }

        None
    }

    fn present_viewport(&mut self, ctx: &mut I::Context) -> presentation::Viewport<I::Texture> {
        let mut layers = Vec::new();

        let composite = self.compositor.render(ctx);
        layers.push(presentation::Layer::Texture(composite));

        if let Some(stroke) = &mut self.state.brush_stroke {
            let texture = stroke.render(ctx);
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

    fn present_color_picker(&mut self) -> presentation::ColorPicker {
        presentation::ColorPicker::OkhsvHueSlice { hue: 0.0 }
    }
}
