#![feature(once_cell_get_mut)]

mod app;
mod config;
mod plugin;
mod renderer;
mod ui;
mod window;

use crate::app::App;
use crate::ui::menu;
use crate::window::Render;
use cpal::traits::DeviceTrait;
use log::info;
use tracing_subscriber::fmt::time::LocalTime;
use voxea_alloc::perf;
use voxea_alloc::perf::PerfTrace;
use winit::application::ApplicationHandler;
use winit::event_loop::EventLoop;

#[global_allocator]
static GLOBAL: voxea_alloc::MemAllocator = voxea_alloc::MemAllocator::new();

fn main() {
    perf::init();

    perf::begin_perf!("main");

    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .init();

    let event_loop = EventLoop::builder()
        .build()
        .expect("Could not create event loop!");

    let app = App::new();
    app.run(event_loop, |cx, event_loop| {
        menu::init(cx, event_loop);
        plugin::run_plugins().unwrap();
    });

    info!("Bye bye!");
}
