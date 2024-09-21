use crate::renderer::RenderContext;
use crate::window::{Render, Window, WindowContext};
use anyhow::Result;
use log::{error, warn};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};
use voxea_alloc::perf;
use voxea_alloc::perf::PerfTrace;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::windows::WindowExtWindows;
use winit::window::{WindowAttributes, WindowId};

#[derive(Default)]
pub struct App {
    pub(crate) windows: FxHashMap<WindowId, Option<Window>>,
    pub(crate) on_start_callback: Option<Box<dyn FnOnce(&mut App, &ActiveEventLoop)>>,
    pub(crate) wait_cancelled: bool,
    pub(crate) render_context: Option<RenderContext>,
}

const WAIT_TIME: Duration = Duration::from_micros(16666);

impl App {
    pub fn new() -> Self {
        Self {
            windows: FxHashMap::default(),
            on_start_callback: None,
            wait_cancelled: false,
            render_context: None,
        }
    }

    pub fn open_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_attributes: Option<WindowAttributes>,
        view: Option<Box<dyn Render + 'static>>,
        backend: bool,
    ) -> Result<&Option<Window>> {
        perf::begin_perf!("app::open_window");

        let mut window = Window::new(event_loop, window_attributes, view, backend)?;

        if let Some(mut view) = window.view.take() {
            let mut cx = WindowContext {
                app: self,
                window: &mut window,
            };

            view.on_open(&mut cx);

            let _ = window.view.insert(view);
        }

        let id = window.window.id();
        self.windows.insert(id, Some(window));

        Ok(self.windows.get(&id).unwrap())
    }

    pub fn get_window(&mut self, window_id: &WindowId) -> Option<&mut Window> {
        let window = self.windows.get_mut(window_id);

        window?.as_mut()
    }

    pub fn set_enable_all_other_windows(&mut self, keep: &WindowId, enable: bool) {
        for window in self.windows.values_mut() {
            if window.is_some() && &window.as_ref().unwrap().window.id() != keep {
                error!("{:?}", window.as_ref().unwrap().window);
                window.as_mut().unwrap().window.set_enable(enable);
            }
        }
        // self.windows
        //     .into_iter()
        //     .filter_map(|mut w| {
        //         if &w.0 != keep {
        //             w.1.as_mut()
        //         } else {
        //             None
        //         }
        //     })
        //     .for_each(|w| w.window.set_enable(enable));
    }

    pub fn run<F>(mut self, event_loop: EventLoop<()>, on_start_callback: F)
    where
        F: FnOnce(&mut App, &ActiveEventLoop) + 'static,
    {
        self.on_start_callback = Some(Box::new(on_start_callback));

        let _ = event_loop.run_app(&mut self);
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. });
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Relinquishes ownership since it should only be called once
        if let Some(on_start_callback) = self.on_start_callback.take() {
            on_start_callback(self, event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        perf::begin_perf!("app::window_event");

        // Moves ownership of the window to the current scope to avoid double borrowing
        let Some(mut window) = self
            .windows
            .get_mut(&window_id)
            .map_or(&mut None, |w| w)
            .take()
        else {
            warn!("Could not find window with id: {window_id:?}!");
            return;
        };

        let response = window.on_window_event(self, event_loop, &event);

        match event {
            WindowEvent::Resized(size) => {}

            WindowEvent::RedrawRequested => {}

            WindowEvent::CloseRequested => {}

            _ => {}
        }

        if window.running {
            // Moves ownership of the window back to the App
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
            for window in self
                .windows
                .values()
                .filter_map(|w| w.as_ref())
                .filter(|w| w.running)
            {
                window.request_redraw();
            }

            // Waits some specified amount of time, i.e. 16.6ms to achieve 60 FPS frame cap, to reduce CPU usage
            // TODO(@Aliremu): Integrate with monitors refresh rate to achieve VSYNC
            event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + WAIT_TIME));
        }
    }
}
