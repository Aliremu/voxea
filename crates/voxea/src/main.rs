#![feature(once_cell_get_mut)]

mod app;
mod config;
mod plugin;
mod renderer;
mod ui;
mod window;

use crate::app::App;
use crate::ui::menu;
use log::info;
use tracing_subscriber::fmt::time::LocalTime;
use voxea_alloc::perf;
use voxea_alloc::perf::PerfTrace;
use winit::event_loop::EventLoop;
use anyhow::Result;

#[global_allocator]
static GLOBAL: voxea_alloc::MemAllocator = voxea_alloc::MemAllocator::new();

fn main() -> Result<()> {
    perf::init();

    perf::begin_perf!("main");

    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .with_ansi(false)
        .init();

    plugin::init()?;
    renderer::init();

    let event_loop = EventLoop::builder()
        .build()
        .expect("Could not create event loop!");

    let app = App::new();
    app.run(event_loop, |cx, event_loop| {
        menu::init(cx, event_loop);

        std::thread::spawn(|| {
            plugin::load_plugins().unwrap();
        });
    });

    info!("Bye bye!");

    Ok(())
}
