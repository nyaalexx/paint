use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Weak};
use std::thread::JoinHandle;

use paint_core::behaviour::{Action, BrushState, Event};
use paint_core::presentation;
use paint_wgpu::Texture;

use crate::gpu::GpuContext;
use crate::surface::Surface;

pub mod ffi {
    use glam::{Affine2, UVec2, Vec2};
    use jni::JNIEnv;
    use jni::objects::JObject;
    use jni_fn::jni_fn;

    use super::*;

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn create(_env: JNIEnv, _this: JObject, gpu_ptr: usize) -> usize {
        let gpu = unsafe { &*(gpu_ptr as *const GpuContext) };
        let behaviour = Behaviour::new(gpu);
        Box::into_raw(Box::new(behaviour)) as usize
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn setCanvasResolution(_env: JNIEnv, _this: JObject, ptr: usize, width: u32, height: u32) {
        let behaviour = unsafe { &*(ptr as *const Behaviour) };
        let event = Event::SetCanvasResolution(UVec2::new(width, height));
        behaviour.handle_event(event);
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn setViewportTransform(
        _env: JNIEnv,
        _this: JObject,
        ptr: usize,
        scale: f32,
        angle: f32,
        x: f32,
        y: f32,
    ) {
        let behaviour = unsafe { &*(ptr as *const Behaviour) };
        let event = Event::SetViewportTransform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(x, y),
        ));
        behaviour.handle_event(event);
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn beginBrushStroke(_env: JNIEnv, _this: JObject, ptr: usize) {
        let behaviour = unsafe { &*(ptr as *const Behaviour) };
        let event = Event::BeginBrushStroke;
        behaviour.handle_event(event);
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn updateBrushStroke(
        _env: JNIEnv,
        _this: JObject,
        ptr: usize,
        x: f32,
        y: f32,
        pressure: f32,
    ) {
        let behaviour = unsafe { &*(ptr as *const Behaviour) };
        let event = Event::UpdateBrushStroke(BrushState {
            position: Vec2::new(x, y),
            pressure,
        });
        behaviour.handle_event(event);
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn endBrushStroke(_env: JNIEnv, _this: JObject, ptr: usize) {
        let behaviour = unsafe { &*(ptr as *const Behaviour) };
        let event = Event::EndBrushStroke;
        behaviour.handle_event(event);
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn attachViewport(_env: JNIEnv, _this: JObject, ptr: usize, surface_ptr: usize) {
        let behaviour = unsafe { &*(ptr as *const Behaviour) };
        let surface = unsafe { &*(surface_ptr as *const Arc<Surface>) };
        behaviour.attach_viewport(Arc::downgrade(surface));
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn attachColorPicker(_env: JNIEnv, _this: JObject, ptr: usize, surface_ptr: usize) {
        let behaviour = unsafe { &*(ptr as *const Behaviour) };
        let surface = unsafe { &*(surface_ptr as *const Arc<Surface>) };
        behaviour.attach_color_picker(Arc::downgrade(surface));
    }

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.Behaviour$Native")]
    pub fn destroy(_env: JNIEnv, _this: JObject, ptr: usize) {
        unsafe {
            drop(Box::from_raw(ptr as *mut Behaviour));
        }
    }
}

struct Impls;

impl paint_core::behaviour::Impls for Impls {
    type Texture = paint_wgpu::Texture;
    type Context = paint_wgpu::FrameContext;
    type Compositor = paint_wgpu::Compositor;
    type BrushEngine = paint_wgpu::BrushEngine;
    type BrushStroke = paint_wgpu::BrushStroke;
}

type BehaviourImpl = paint_behaviour::Behaviour<Impls>;

#[derive(Debug, Clone)]
enum Command {
    Stop,
    AttachViewport(Weak<Surface>),
    AttachColorPicker(Weak<Surface>),
    HandleEvent(Event),
}

#[derive(Debug)]
pub struct Behaviour {
    thread_handle: Option<JoinHandle<()>>,
    command_sender: Sender<Command>,
}

impl Behaviour {
    pub fn new(gpu: &GpuContext) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();

        let behaviour_thread = BehaviourThread::new(gpu, command_receiver);
        let thread_handle = std::thread::spawn(move || behaviour_thread.run());

        Self {
            thread_handle: Some(thread_handle),
            command_sender,
        }
    }

    fn send_command(&self, command: Command) {
        let _ = self.command_sender.send(command);
    }

    pub fn attach_viewport(&self, surface: Weak<Surface>) {
        self.send_command(Command::AttachViewport(surface));
        self.handle_event(Event::InvalidateViewport);
    }

    pub fn attach_color_picker(&self, surface: Weak<Surface>) {
        self.send_command(Command::AttachColorPicker(surface));
        self.handle_event(Event::InvalidateColorPicker);
    }

    pub fn handle_event(&self, event: Event) {
        self.send_command(Command::HandleEvent(event));
    }
}

impl Drop for Behaviour {
    fn drop(&mut self) {
        self.send_command(Command::Stop);

        if let Some(thread) = self.thread_handle.take() {
            thread.join().unwrap();
        }
    }
}

struct BehaviourThread {
    behaviour_impl: BehaviourImpl,
    command_receiver: Receiver<Command>,
    frame_context: LazyFrameContext,

    viewport_renderer: paint_wgpu::ViewportRenderer,
    viewport_surface: Option<Weak<Surface>>,

    color_picker_renderer: paint_wgpu::ColorPickerRenderer,
    color_picker_surface: Option<Weak<Surface>>,
}

impl BehaviourThread {
    pub fn new(gpu: &GpuContext, command_receiver: Receiver<Command>) -> Self {
        let context = gpu.context.clone();
        let compositor = paint_wgpu::Compositor::new(context.clone());
        let brush_engine = paint_wgpu::BrushEngine::new(context.clone());
        let behaviour_impl = BehaviourImpl::new(compositor, brush_engine);

        let viewport_renderer = paint_wgpu::ViewportRenderer::new(context.clone());
        let color_picker_renderer = paint_wgpu::ColorPickerRenderer::new(context.clone());

        Self {
            behaviour_impl,
            command_receiver,
            frame_context: LazyFrameContext::new(gpu.context.clone()),
            viewport_renderer,
            viewport_surface: None,
            color_picker_renderer,
            color_picker_surface: None,
        }
    }

    pub fn run(mut self) {
        while let Ok(cmd) = self.command_receiver.recv() {
            self.handle_command(cmd);

            while let Ok(cmd) = self.command_receiver.try_recv() {
                self.handle_command(cmd);
            }

            self.perform_actions();
        }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Stop => {}

            Command::HandleEvent(event) => {
                tracing::trace!("Handling event: {event:?}");
                let ctx = self.frame_context.get_mut();
                self.behaviour_impl.handle_event(ctx, event);
            }

            Command::AttachViewport(surface) => {
                self.viewport_surface = Some(surface);
                tracing::trace!("Attached viewport surface");
            }

            Command::AttachColorPicker(surface) => {
                self.color_picker_surface = Some(surface);
                tracing::trace!("Attached color picker surface");
            }
        }

        // TODO: remove, this is just for debugging
        let ctx = self.frame_context.get_mut();
        self.behaviour_impl
            .handle_event(ctx, Event::InvalidateColorPicker);
    }

    fn perform_actions(&mut self) {
        loop {
            let ctx = self.frame_context.get_mut();
            let Some(action) = self.behaviour_impl.perform_action(ctx) else {
                return;
            };

            match action {
                Action::PresentViewport(viewport) => self.present_viewport(&viewport),
                Action::PresentColorPicker(color_picker) => {
                    self.present_color_picker(&color_picker)
                }
            }
        }
    }

    fn present_viewport(&mut self, viewport: &presentation::Viewport<Texture>) {
        let Some(surface) = self.viewport_surface.as_ref().and_then(Weak::upgrade) else {
            return;
        };

        surface.render(|target| {
            let ctx = self.frame_context.take();
            self.viewport_renderer.render(ctx, target, viewport);
            tracing::trace!("Rendered viewport");
        });
    }

    fn present_color_picker(&mut self, color_picker: &presentation::ColorPicker) {
        let Some(surface) = self.color_picker_surface.as_ref().and_then(Weak::upgrade) else {
            return;
        };

        surface.render(|target| {
            let ctx = self.frame_context.take();
            self.color_picker_renderer.render(ctx, target, color_picker);
            tracing::trace!("Rendered color picker");
        });
    }
}

struct LazyFrameContext {
    global_context: Arc<paint_wgpu::GlobalContext>,
    frame_context: Option<paint_wgpu::FrameContext>,
}

impl LazyFrameContext {
    pub fn new(global_context: Arc<paint_wgpu::GlobalContext>) -> Self {
        Self {
            global_context,
            frame_context: None,
        }
    }
}

impl LazyFrameContext {
    pub fn get_mut(&mut self) -> &mut paint_wgpu::FrameContext {
        self.frame_context
            .get_or_insert_with(|| paint_wgpu::FrameContext::new(&self.global_context))
    }

    pub fn take(&mut self) -> paint_wgpu::FrameContext {
        self.frame_context
            .take()
            .unwrap_or(paint_wgpu::FrameContext::new(&self.global_context))
    }
}
