use std::ffi::c_void;
use std::sync::Arc;
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
use voxea_vst::base::funknown::{IPlugViewContentScaleSupport_Impl, IPlugView_Impl};
use voxea_vst::VSTHostContext;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop, plug: u32) {
    let window_attributes = WindowAttributes::default()
        .with_title("Plugin")
        .with_inner_size(PhysicalSize::new(1200, 800));

    let vst = voxea_vst::load_vst(plug);
    let plugin = PlugView {
        vst
    };

    // let plugin = PlugView::default();

    let window = cx.open_window(
        event_loop,
        Some(window_attributes),
        Some(Box::new(plugin)),
    )
    .expect("Failed to open plugin view");

    // let window_handle = window.as_ref().unwrap().window.window_handle().unwrap().as_raw();
    //
    // let hwnd = match window_handle {
    //     RawWindowHandle::Win32(handle) => handle.hwnd.get(),
    //     _ => todo!("Not running on Windows"),
    // };
    //
    // let vst = voxea_vst::load_vst_window(hwnd as HWND, plug);
}

#[derive(Default)]
pub struct PlugView {
    vst: Arc<VSTHostContext>
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Render for PlugView {
    fn on_open(&mut self, cx: &mut WindowContext) {
        let window_handle = cx.window.window.window_handle().unwrap().as_raw();

        let hwnd = match window_handle {
            RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => todo!("Not running on Windows"),
        };

        self.vst.attach(hwnd as HWND);
        let mut view = (*(self.vst)).view;
        // unsafe { (*view).(2.5); }
    }

    fn window_event(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                let rect = Box::new(Rect {
                    left: 0,
                    top: 0,
                    right: size.width as i32,
                    bottom: size.height as i32
                });

                unsafe {
                    // let view = (*(self.vst)).view;

                    // (*(view)).on_size(Box::into_raw(rect) as *const c_void);
                }

            }

            _ => {}
        };
    }

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {
    }
}
