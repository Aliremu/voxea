use crate::window::{Render, WindowContext};
use crate::App;
use cpal::traits::{HostTrait, StreamTrait};
use log::warn;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rodio::DeviceTrait;
use std::ffi::c_void;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use voxea_audio::vst::host::{HostParameterChanges, VSTHostContext};
use voxea_vst::base::funknown::{IAudioProcessor_Impl, IPlugView_Impl};
use voxea_vst::gui::plug_view::{PlatformType, ViewRect};
use voxea_vst::vst::audio_processor::{
    AudioBusBuffers, ProcessData, ProcessMode, SymbolicSampleSize,
};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::WindowAttributes;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop, plug: u32) {
    let window_attributes = WindowAttributes::default()
        .with_title("Plugin")
        .with_inner_size(PhysicalSize::new(1200, 800));

    let plugin = PlugView {
        tx: None,
    };

    let _ = cx
        .open_window(
            event_loop,
            Some(window_attributes),
            Some(Box::new(plugin)),
            false,
        )
        .expect("Failed to open plugin view");
}

pub enum PluginCommand {
    CloseWindow,
}

pub struct PlugView {
    tx: Option<mpsc::Sender<PluginCommand>>,
}

impl Render for PlugView {
    fn on_open(&mut self, cx: &mut WindowContext) {
        let window_handle = cx.window.window.window_handle().unwrap().as_raw();

        let hwnd = match window_handle {
            RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => todo!("Not running on Windows"),
        };

        unsafe {
            cx.app.audio_engine.plugin_modules.read()
                .expect("Could not read plugin modules!")
                .get(0)
                .expect("Could not get plugin 0!")
                .view
                .unwrap()
                .attached(hwnd as *mut c_void, PlatformType::HWND);
        }
    }

    fn window_event(
        &mut self,
        cx: &mut WindowContext,
        _event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.tx.as_ref().inspect(|tx| {
                    let _ = tx.send(PluginCommand::CloseWindow);
                });

                unsafe {
                    cx.app.audio_engine.plugin_modules.read()
                        .expect("Could not read plugin modules!")
                        .get(0)
                        .expect("Could not get plugin 0!")
                        .view
                        .unwrap()
                        .removed();
                }
            }

            WindowEvent::Resized(_size) => {
                unsafe {
                    let view = cx.app.audio_engine.plugin_modules.read().unwrap().get(0).unwrap()
                        .view
                        .unwrap();
                    let mut rect = ViewRect::default();
                    view.check_size_constraint(&mut rect);
                    let _ = cx.window
                        .window
                        .request_inner_size(PhysicalSize::new(rect.right, rect.bottom));
                    warn!("SIZE CONSTRAINTS: {:?}", rect);
                }
            }

            _ => {}
        };
    }

    fn render(&mut self, _cx: &mut WindowContext, _event_loop: &ActiveEventLoop) {}
}
