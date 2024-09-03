use crate::renderer::RenderContext;
use crate::ui::menu;
use crate::App;
use anyhow::Result;
use egui::ViewportBuilder;
use egui_winit::{apply_viewport_builder_to_window, create_winit_window_attributes};
use log::{info, warn};
use std::sync::Arc;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowAttributes;
use winit::{dpi::PhysicalSize, window::Window as WinitWindow};

pub struct WindowContext<'a> {
    pub(crate) app: &'a mut App,
    pub(crate) window: &'a mut Window,
}

pub trait Render: 'static {
    fn window_event(
        &mut self,
        cx: &mut WindowContext,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) {
    }
    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop);
}

/// A wrapper around the winit window
pub struct Window {
    pub(crate) window: Arc<WinitWindow>,
    pub(crate) cx: RenderContext,
    pub(crate) view: Option<Box<dyn Render + 'static>>,
    pub(crate) running: bool,

    // winit bug where random resize events are sent on startup. https://github.com/rust-windowing/winit/issues/2094
    pub(crate) init: bool,
}

impl Window {
    /// Creates a window and initializes the [`RenderContext`] and egui states
    /// Must be called in the main thread to have access to [`ActiveEventLoop`]
    pub fn new(
        event_loop: &ActiveEventLoop,
        window_attributes: Option<WindowAttributes>,
        view: Option<Box<dyn Render + 'static>>,
    ) -> Result<Self> {
        let egui_ctx = egui::Context::default();
        egui_ctx.set_fonts(egui::FontDefinitions::default());
        egui_ctx.set_style(egui::Style::default());

        let viewport_builder = ViewportBuilder::default();

        let window_attributes = window_attributes.map_or(
            create_winit_window_attributes(&egui_ctx, event_loop, viewport_builder.clone()),
            |v| v,
        );

        let window = Arc::new(event_loop.create_window(window_attributes)?);
        apply_viewport_builder_to_window(&egui_ctx, &window, &viewport_builder);

        let cx = RenderContext::new(egui_ctx, window.clone());

        Ok(Self {
            window,
            cx,
            view,
            running: true,
            init: false,
        })
    }

    #[inline]
    pub fn on_window_event(
        &mut self,
        cx: &mut App,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        let response = self.cx.egui_state.on_window_event(&self.window, event);

        if let Some(mut view) = self.view.take() {
            let mut cx = WindowContext {
                app: cx,
                window: self,
            };

            view.window_event(&mut cx, event_loop, &event);

            self.view = Some(view);
        }

        match event {
            WindowEvent::CloseRequested => {
                self.running = false;
            }

            WindowEvent::Resized(size) => {
                if self.init {
                    info!("Resizing window to {size:?}");
                    self.resize(size);
                } else {
                    self.init = true;
                }
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
                    menu::init(cx, event_loop);
                }
            }

            WindowEvent::RedrawRequested => {
                // Redraw the UI if egui deems it necessary, eg. state has been updated
                if response.repaint {
                    if let Some(mut view) = self.view.take() {
                        let mut cx = WindowContext {
                            app: cx,
                            window: self,
                        };

                        view.render(&mut cx, event_loop);

                        self.view = Some(view);
                    }
                }
            }
            _ => {}
        }

        response
    }

    #[inline]
    pub fn resize(&mut self, physical_size: &PhysicalSize<u32>) {
        self.cx.resize(physical_size);
    }

    #[inline]
    pub fn render(&mut self, cx: &mut App) {
        self.cx.render(&self.window);
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        warn!("Dropping window! {:?}", self.window.id());
    }
}
