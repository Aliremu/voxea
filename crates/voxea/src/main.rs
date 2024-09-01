mod renderer;
mod window;
mod ui;
mod app;
mod config;

use cpal::traits::DeviceTrait;
use log::info;
use winit::application::ApplicationHandler;
use winit::event_loop::{EventLoop};
use tracing_subscriber::fmt::time::LocalTime;

use crate::window::{Render};
use crate::app::App;
use crate::ui::menu;

fn main() {
    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .init();

    let event_loop = EventLoop::builder()
        .build()
        .expect("Could not create event loop!");

    let mut app = App::new();
    app.run(event_loop, |cx, event_loop| {
        menu::init(cx, event_loop);
        // voxea_audio::enumerate_input_devices(host);
    });
}
