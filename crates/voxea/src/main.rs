mod renderer;
mod window;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window as WinitWindow, WindowId};

use egui::{Key, ViewportBuilder};
use egui_wgpu::wgpu;
use log::{info, warn};
use std::collections::HashMap;
use std::hash::RandomState;
use std::time::{Duration, Instant};
use std::{borrow::Cow, sync::Arc};
use tracing_subscriber::fmt::time::LocalTime;

use crate::window::Window;
use anyhow::Result;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    windows: HashMap<WindowId, Option<Window>, RandomState>,
    wait_cancelled: bool,
    text: String,
}

const WAIT_TIME: Duration = Duration::from_micros(16666);

impl App {
    pub fn open_window(&mut self, event_loop: &ActiveEventLoop) -> Result<WindowId> {
        let window = Window::new(event_loop)?;

        let id = window.window.id();
        self.windows.insert(id, Some(window));

        Ok(id)
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. });
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        voxea_audio::enumerate_hosts().unwrap();

        self.open_window(event_loop).unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Moves ownership of the window to the current scope to avoid breaking borrow rules
        let Some(mut window) = self
            .windows
            .get_mut(&window_id)
            .map_or(&mut None, |w| w)
            .take()
        else {
            warn!("Could not find window with id: {window_id:?}!");
            return;
        };

        let inner_size = window
            .window
            .inner_size()
            .to_logical::<f32>(window.window.scale_factor());

        let width = inner_size.width;
        let height = inner_size.height;

        let response = window.on_window_event(self, event_loop, &event);

        match event {
            WindowEvent::Resized(size) => {
                info!("Resizing window to {size:?}");
                window.resize(size);
            }

            WindowEvent::RedrawRequested => {
                // Redraw the UI if egui deems it necessary, eg. state has been updated
                if response.repaint {
                    window.cx.ui(move |cx| {
                        egui::SidePanel::left("left_panel")
                            .resizable(true)
                            .default_width(width / 3.0)
                            .max_width(width)
                            .show(cx, |ui| {
                                let button = ui.button("Click me!");

                                let mut lol = "HELLO WORLD";
                                ui.separator();

                                if button.clicked() {
                                    info!("Clicked!");
                                    std::thread::spawn(|| {
                                        voxea_audio::beep().unwrap();
                                    });
                                }
                            });
                    });
                }

                window.render(self);
            }

            WindowEvent::CloseRequested => {}

            _ => {}
        }

        if window.running {
            let _ = self.windows.insert(window_id, Some(window));
        } else {
            // Drops the reference to the window which closes it
            self.windows.remove(&window_id);
        }

        // Exits the event loop when all windows have been closed
        if self.windows.is_empty() {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.wait_cancelled {
            for window in self.windows.values() {
                window.as_ref().unwrap().window.request_redraw();
            }

            event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + WAIT_TIME));
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .init();

    let event_loop = EventLoop::builder()
        .build()
        .expect("Could not create event loop!");

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
