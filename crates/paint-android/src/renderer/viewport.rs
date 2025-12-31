use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use paint_core::presentation::Viewport;

use super::Renderer;
use crate::gpu::GpuContext;

pub mod ffi {
    use jni::JNIEnv;
    use jni::objects::JObject;
    use jni_fn::jni_fn;

    use super::*;

    #[unsafe(no_mangle)]
    #[jni_fn("site.nyaalex.paint.rust.ViewportRenderer$Native")]
    pub fn new(_env: JNIEnv, _this: JObject, gpu_ptr: usize) -> usize {
        let gpu = unsafe { &*(gpu_ptr as *const GpuContext) };

        let renderer = ViewportRenderer::new(gpu);
        let renderer: Arc<dyn Renderer> = Arc::new(renderer);
        Box::into_raw(Box::new(renderer)) as usize
    }
}

#[derive(Debug)]
enum Command {
    Interrupt,
    Present(paint_wgpu::FrameContext, Viewport<paint_wgpu::Texture>),
}

#[derive(Debug)]
pub struct ViewportRenderer {
    inner: Mutex<Inner>,
    command_sender: Sender<Command>,
}

impl ViewportRenderer {
    pub fn new(gpu: &GpuContext) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        let renderer = paint_wgpu::ViewportRenderer::new(gpu.context.clone());

        Self {
            inner: Mutex::new(Inner {
                context: gpu.context.clone(),
                renderer,
                command_receiver,
                viewport: None,
                frame_context: None,
            }),
            command_sender,
        }
    }

    fn send_command(&self, command: Command) {
        let _ = self.command_sender.send(command);
    }

    pub fn present(
        &self,
        frame_context: paint_wgpu::FrameContext,
        viewport: Viewport<paint_wgpu::Texture>,
    ) {
        self.send_command(Command::Present(frame_context, viewport));
    }
}

impl Renderer for ViewportRenderer {
    fn interrupt(&self) {
        let _ = self.command_sender.send(Command::Interrupt);
    }

    fn update(&self) {
        let mut inner = self.inner.lock().unwrap();

        let Ok(command) = inner.command_receiver.recv() else {
            return;
        };

        match command {
            Command::Interrupt => {}
            Command::Present(frame_context, viewport) => {
                inner.viewport = Some(viewport);
                inner.frame_context = Some(frame_context);
            }
        }
    }

    fn render(&self, texture: &wgpu::Texture) {
        let mut inner = self.inner.lock().unwrap();
        inner.render(texture);
    }
}

#[derive(Debug)]
struct Inner {
    context: Arc<paint_wgpu::Context>,
    renderer: paint_wgpu::ViewportRenderer,
    viewport: Option<Viewport<paint_wgpu::Texture>>,
    frame_context: Option<paint_wgpu::FrameContext>,
    command_receiver: Receiver<Command>,
}

impl Inner {
    pub fn render(&mut self, texture: &wgpu::Texture) {
        let Some(viewport) = &self.viewport else {
            return;
        };

        let frame_context = self
            .frame_context
            .take()
            .unwrap_or_else(|| paint_wgpu::FrameContext::new(&self.context));

        self.renderer.render(frame_context, texture, viewport);
    }
}
