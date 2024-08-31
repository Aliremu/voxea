use egui::ViewportBuilder;
use std::sync::Arc;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::{dpi::PhysicalSize, window::Window as WinitWindow};

use crate::renderer::RenderContext;
use crate::App;

pub struct Window {
    pub(crate) window: Arc<WinitWindow>,
    pub(crate) cx: RenderContext,
    pub(crate) running: bool,
}

impl Window {
    pub fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        let egui_ctx = egui::Context::default();
        egui_ctx.set_fonts(egui::FontDefinitions::default());
        egui_ctx.set_style(egui::Style::default());

        let window = Arc::new(egui_winit::create_window(
            &egui_ctx,
            event_loop,
            &ViewportBuilder::default(),
        )?);

        let cx = RenderContext::new(egui_ctx, window.clone());

        Ok(Self {
            window,
            cx,
            running: true,
        })
    }

    #[inline]
    pub fn on_window_event(
        &mut self,
        cx: &mut App,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        match event {
            WindowEvent::CloseRequested => {
                self.running = false;
            }

            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if !is_synthetic
                    && !event.repeat
                    && event.state == ElementState::Pressed
                    && event.physical_key == KeyCode::KeyG
                {
                    cx.open_window(event_loop).unwrap();
                }
            }
            _ => {}
        }

        let response = self.cx.egui_state.on_window_event(&self.window, event);

        response
    }

    #[inline]
    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.cx.resize(physical_size);
    }
    #[inline]
    pub fn render(&mut self, cx: &mut App) {
        self.cx.render(&self.window);
    }
}
