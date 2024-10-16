#![feature(once_cell_get_mut)]

mod app;
mod config;
mod renderer;
mod ui;
mod window;

use std::fs;

use crate::app::App;
use crate::ui::menu;
use anyhow::Result;
use log::info;
use tracing_subscriber::fmt::time::LocalTime;
use voxea_alloc::perf;
use voxea_alloc::perf::PerfTrace;
use winit::event_loop::EventLoop;
use winit::platform::windows::EventLoopBuilderExtWindows;

#[global_allocator]
static GLOBAL: voxea_alloc::MemAllocator = voxea_alloc::MemAllocator::new();

fn main() -> Result<()> {
    perf::init();

    perf::begin_perf!("main");

    tracing_subscriber::fmt()
        .with_timer(LocalTime::rfc_3339())
        .init();

    renderer::init();

    let event_loop = EventLoop::builder()
        .with_dpi_aware(true)
        .build()
        .expect("Could not create event loop!");

    let app = App::new();
    app.run(event_loop, |cx, event_loop| {
        cx.audio_engine.run();

        let paths = fs::read_dir("C:/Coding/RustRover/voxea/vst3/WithUI")
            .expect("Could not read plugins directory!");

        // Checks for all .vert, .frag, or .comp files in shaders directory and compiles them to SPIR-V
        paths
            .into_iter()
            .filter_map(|p| p.ok())
            .map(|p| p.path())
            .for_each(|path| {
                if path.extension().map_or(false, |ext| ext == "vst3") {
                    // let parent = path.parent().unwrap();
                    // let name = path.file_name().unwrap().to_str().unwrap();

                    cx.audio_engine.add_plugin(path.to_str().unwrap());
                }
            });

        menu::init(cx, event_loop);
    });

    info!("Bye bye!");

    Ok(())
}
