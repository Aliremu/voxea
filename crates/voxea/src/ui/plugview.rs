use crate::ui::settings;
use crate::window::{Render, WindowContext};
use crate::{plugin, renderer, App};
use egui::{pos2, Color32};
use egui::load::SizedTexture;
use log::info;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::platform::windows::{WindowExtWindows, HWND};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::WindowAttributes;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop) {
    let window_attributes = WindowAttributes::default()
        .with_title("Plugin")
        .with_inner_size(PhysicalSize::new(1200, 800));

    let window = cx.open_window(
        event_loop,
        Some(window_attributes),
        Some(Box::new(Menu::default())),
    )
        .expect("Failed to open menu");
    let window_handle = window.as_ref().unwrap().window.window_handle().unwrap().as_raw();

    let hwnd = match window_handle {
        RawWindowHandle::Win32(handle) => handle.hwnd.get(),
        _ => todo!("Not running on Windows"),
    };

    voxea_vst::load_vst(hwnd as HWND);
}

#[derive(Default)]
pub struct Menu {}

impl Render for Menu {
    fn window_event(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop, event: &WindowEvent) {
    }

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {
    }
}
