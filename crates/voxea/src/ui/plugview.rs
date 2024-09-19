use crate::ui::settings;
use crate::window::{Render, WindowContext};
use crate::{plugin, renderer, App};
use egui::load::SizedTexture;
use egui::{pos2, Color32};
use log::info;
use voxea_vst::vst::audio_processor::{AudioBusBuffers, HostParameterChanges, ProcessData, ProcessMode, SymbolicSampleSize};
use std::ffi::c_void;
use std::sync::Arc;
use std::time::Duration;
use voxea_vst::base::funknown::{IAudioProcessor_Impl, IPlugViewContentScaleSupport_Impl, IPlugView_Impl};
use voxea_vst::VSTHostContext;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::platform::windows::{WindowExtWindows, HWND};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::WindowAttributes;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop, plug: u32) {
    let window_attributes = WindowAttributes::default()
        .with_title("Plugin")
        .with_inner_size(PhysicalSize::new(1200, 800));

    let vst = voxea_vst::load_vst(plug).expect("Couldn't load VST!");
    let plugin = PlugView { vst };

    // let plugin = PlugView::default();

    let window = cx
        .open_window(event_loop, Some(window_attributes), Some(Box::new(plugin)))
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
    vst: Arc<VSTHostContext>,
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
        
        let vst = self.vst.clone();

        std::thread::spawn(move || {
            unsafe {
                let mut processor = *(vst.processor);
                let mut count = 0;

                const block_size: usize = 128;

                loop {
                    processor.set_processing(true);

                    let mut data1 = [0.25f32; block_size];
                    let mut data2 = [0.0f32; block_size];

                    let mut inputs: Vec<Option<&[f32; block_size]>> = vec![Some(&data1); 1];
                    let mut outputs: Vec<Option<&[f32; block_size]>> = vec![Some(&data2); 1];

                    let mut in_bus = AudioBusBuffers {
                        num_channels: 1,
                        silence_flags: 0,
                        channel_buffers_32: inputs.as_mut_ptr() as *mut _,
                    };

                    let mut out_bus = AudioBusBuffers {
                        num_channels: 1,
                        silence_flags: 0,
                        channel_buffers_32: outputs.as_mut_ptr() as *mut _,
                    };

                    // let mut out: *mut c_void = std::ptr::null_mut();
                    let mut data = ProcessData {
                        process_mode: ProcessMode::Realtime,
                        symbolic_sample_size: SymbolicSampleSize::Sample32,
                        num_samples: 128,
                        num_inputs: 1,
                        num_outputs: 1,
                        inputs: &mut in_bus,
                        outputs: &mut out_bus,
                        input_parameter_changes: Box::into_raw(HostParameterChanges::new()) as *mut _,
                        output_parameter_changes: None,
                        input_events: None,
                        output_events: None,
                        process_context: None,
                    };

                    //let data = Arc::new(ProcessData::prepare(out, &inputs, &outputs));

                    processor.process(&mut data);
                    count += 1;
                    // let arr: &[*mut f32] = unsafe { std::slice::from_raw_parts(, 1) };
                    // let arr: &[f32] = unsafe { std::slice::from_raw_parts(arr[0], block_size) };
                    info!("{:?}; ProcessData: {:?}", count, &mut outputs);

                    processor.set_processing(false);

                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        });
    }

    fn window_event(
        &mut self,
        cx: &mut WindowContext,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                let rect = Box::new(Rect {
                    left: 0,
                    top: 0,
                    right: size.width as i32,
                    bottom: size.height as i32,
                });

                unsafe {
                    // let view = (*(self.vst)).view;

                    // (*(view)).on_size(Box::into_raw(rect) as *const c_void);
                }
            }

            _ => {}
        };
    }

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {}
}
