mod persistence_manager;

use std::sync::{Arc, Mutex};

use async_executor::LocalExecutor;
use futures_lite::future;
use glam::{Affine2, UVec2};
use paint_core::behaviour::{
    Action, BrushEngine, BrushStroke, Compositor, Event, Impls, StrokeSettings, Texture,
};
use paint_core::presentation;

use crate::persistence_manager::{PersistenceManager, ProjectId};

pub struct Behaviour<I: Impls> {
    async_executor: async_executor::LocalExecutor<'static>,
    shared: Arc<Shared<I>>,

    compositor: I::Compositor,
    brush_engine: I::BrushEngine,

    cur_project: ProjectId,
    viewport_dirty: bool,
    canvas_resolution: UVec2,
    viewport_transform: Affine2,
    brush_stroke: Option<I::BrushStroke>,
}

struct Shared<I: Impls> {
    persistence: PersistenceManager<I>,
    task_queue: TaskQueue<I>,
}

impl<I: Impls> Behaviour<I> {
    pub fn new(compositor: I::Compositor, brush_engine: I::BrushEngine) -> Self {
        let async_executor = LocalExecutor::new();

        let shared = Arc::new(Shared {
            persistence: PersistenceManager::new(),
            task_queue: TaskQueue::new(),
        });

        let canvas_resolution = UVec2::new(2304, 1440);

        let cur_project = future::block_on(shared.persistence.create_project(
            paint_core::persistence::NewProject {
                resolution: canvas_resolution,
            },
        ))
        .unwrap();

        Self {
            async_executor,
            shared,
            compositor,
            brush_engine,

            cur_project,
            viewport_dirty: true,
            canvas_resolution,
            viewport_transform: Affine2::IDENTITY,
            brush_stroke: None,
        }
    }

    pub fn handle_event(&mut self, ctx: &mut I::Context, event: Event) {
        match event {
            Event::InvalidateViewport => {
                self.viewport_dirty = true;
            }

            Event::SetCanvasResolution(resolution) => {
                self.canvas_resolution = resolution;
                self.viewport_dirty = true;
                // TODO: update project
            }

            Event::SetViewportTransform(transform) => {
                self.viewport_transform = transform;
                self.viewport_dirty = true;
            }

            Event::BeginBrushStroke => {
                self.brush_stroke = Some(self.brush_engine.begin_stroke(&StrokeSettings {
                    canvas_resolution: self.canvas_resolution,
                }));
            }

            Event::UpdateBrushStroke(state) => {
                if let Some(stroke) = &mut self.brush_stroke {
                    stroke.update(&state);
                    self.viewport_dirty = true;
                }
            }

            Event::EndBrushStroke => {
                if let Some(mut stroke) = self.brush_stroke.take() {
                    let stroke_texture = stroke.render(ctx);
                    self.compositor.put_texture(ctx, stroke_texture);
                    self.viewport_dirty = true;
                }
            }

            Event::Save(path) => {
                let composite = self.compositor.get_composite(ctx).download(ctx);

                let shared = self.shared.clone();
                let project_id = self.cur_project;

                self.async_executor
                    .spawn(async move {
                        let composite = composite.await;

                        let diff = persistence_manager::Diff {
                            new_texture: composite,
                        };

                        let res = shared
                            .persistence
                            .save_project(project_id, diff, path)
                            .await;
                        res.unwrap(); // TODO: propagate to user through Action

                        tracing::info!("Saved!");
                    })
                    .detach();
            }

            Event::Open(path) => {
                let shared = self.shared.clone();

                self.async_executor
                    .spawn(async move {
                        let project = shared.persistence.open_project(path).await.unwrap();
                        // TODO: propagate error to user through Action

                        shared.task_queue.defer(move |behaviour, ctx| {
                            behaviour.cur_project = project.id;
                            let texture = I::Texture::upload(ctx, project.texture);
                            behaviour.compositor.put_texture(ctx, texture);
                            // TODO: this should include more updates..
                        });

                        tracing::info!("Loaded!");
                    })
                    .detach();
            }
        }
    }

    pub fn update(&mut self, ctx: &mut I::Context) {
        self.async_executor.try_tick();
        self.shared.clone().task_queue.run_all_tasks(self, ctx);
    }

    pub fn perform_action(&mut self, ctx: &mut I::Context) -> Option<Action<I>> {
        if self.viewport_dirty {
            let viewport = self.present_viewport(ctx);
            self.viewport_dirty = false;
            return Some(Action::PresentViewport(viewport));
        }

        None
    }

    fn present_viewport(&mut self, ctx: &mut I::Context) -> presentation::Viewport<I::Texture> {
        let mut layers = Vec::new();

        let composite = self.compositor.get_composite(ctx);
        layers.push(presentation::Layer::Texture(composite));

        if let Some(stroke) = &mut self.brush_stroke {
            let texture = stroke.render(ctx);
            layers.push(presentation::Layer::Texture(texture));
        }

        presentation::Viewport {
            transform: self.viewport_transform,
            canvas: presentation::Canvas {
                resolution: self.canvas_resolution,
                layers,
            },
        }
    }
}

struct TaskQueue<I: Impls> {
    queue: Mutex<Vec<Box<dyn FnOnce(&mut Behaviour<I>, &mut I::Context)>>>,
}

impl<I: Impls> TaskQueue<I> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(Vec::new()),
        }
    }

    pub fn defer<T>(&self, task: T)
    where
        T: FnOnce(&mut Behaviour<I>, &mut I::Context) + 'static,
    {
        self.queue.lock().unwrap().push(Box::new(task));
    }

    pub fn run_all_tasks(&self, behaviour: &mut Behaviour<I>, ctx: &mut I::Context) {
        let mut queue = self.queue.lock().unwrap();
        for task in queue.drain(..) {
            task(behaviour, ctx)
        }
    }
}
