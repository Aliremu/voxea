use crate::vst::host::VSTHostContext;
use crate::window::{Render, WindowContext};
use crate::App;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, Sample};
use log::{info, warn};
use rodio::buffer::SamplesBuffer;
use voxea_vst::vst::audio_processor::{AudioBusBuffers, HostParameterChanges, ProcessData, ProcessMode, SymbolicSampleSize};
use core::{error, panic};
use std::ffi::c_void;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use voxea_vst::base::funknown::IAudioProcessor_Impl;
// use voxea_vst::VSTHostContext;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::HWND;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::WindowAttributes;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop, plug: u32) {
    let window_attributes = WindowAttributes::default()
        .with_title("Plugin")
        .with_inner_size(PhysicalSize::new(1200, 800));

    // let vst = voxea_vst::load_vst(plug).expect("Couldn't load VST!");
    //
    //
    let vst = VSTHostContext::new("C:/Coding/RustRover/voxea/vst3/Archetype Nolly.vst3").unwrap();

    
    let plugin = PlugView { 
        vst: Arc::new(vst), 
        tx: None 
    };

    let window = cx
        .open_window(event_loop, Some(window_attributes), Some(Box::new(plugin)))
        .expect("Failed to open plugin view");
}

pub enum PluginCommand {
    CloseWindow
}

#[derive(Default)]
pub struct PlugView {
    vst: Arc<VSTHostContext>,
    tx: Option<mpsc::Sender<PluginCommand>>
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
        
        #[cfg(feature = "")]
        {
        self.vst.attach(hwnd as HWND);
        let view = (*(self.vst)).view;
        // unsafe { (*view).(2.5); }
        
        let vst = self.vst.clone();
        let (tx, rx) = mpsc::channel();
        
        self.tx = Some(tx);

        std::thread::spawn(move || {
            unsafe {
                let mut processor = *(vst.processor);
                let mut count = 0;

                let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                // let sink = rodio::Sink::try_new(&stream_handle).unwrap();

                const block_size: usize = 192;

                const SAMPLE_RATE: f32 = 44100.0;
                const BLOCK_SIZE: usize = 192;
                const FREQUENCY: f32 = 440.0;
                let phase_step = 2.0 * std::f32::consts::PI * FREQUENCY / SAMPLE_RATE;
                let mut phase: f32 = 0.0;
                let data2 = [0.0f32; block_size];
                let start = Instant::now();

                loop {
                    match rx.try_recv() {
                        Ok(msg) => {
                            panic!();
                            warn!("Window closed!");
                        }

                        Err(err) => {
                            // processor.set_processing(true);

                            let data1: [f32; BLOCK_SIZE] = std::array::from_fn(|_| {
                                // Generate a sine wave using the tracked phase
                                let sample = (phase).sin();
                                phase += phase_step;
                                
                                // Keep phase within the 0 to 2π range to avoid overflow
                                if phase >= 2.0 * std::f32::consts::PI {
                                    phase -= 2.0 * std::f32::consts::PI;
                                }
                                
                                sample
                            });



                            let mut inputs: Vec<Option<&[f32; block_size]>> = vec![Some(&data1); 2];
                            let mut outputs: Vec<Option<&[f32; block_size]>> = vec![Some(&data2); 2];

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

                            let mut out = HostParameterChanges::new();

                            let mut data = ProcessData {
                                process_mode: ProcessMode::Realtime,
                                symbolic_sample_size: SymbolicSampleSize::Sample32,
                                num_samples: 192,
                                num_inputs: 1,
                                num_outputs: 1,
                                inputs: &mut in_bus,
                                outputs: &mut out_bus,
                                input_parameter_changes: Box::into_raw(Box::new(HostParameterChanges::new())) as *mut _,
                                output_parameter_changes: None,
                                input_events: None,
                                output_events: None,
                                process_context: None,
                            };

                            //let data = Arc::new(ProcessData::prepare(out, &inputs, &outputs));

                            // processor.process(&mut data);
                            count += 1;
                            // let arr: &[*mut f32] = unsafe { std::slice::from_raw_parts(, 1) };
                            // let arr: &[f32] = unsafe { std::slice::from_raw_parts(arr[0], block_size) };
                            // info!("{:?}; ProcessData: {:?}", count, &mut outputs);

                            // processor.set_processing(false);
                            let samples = SamplesBuffer::new(1, 44100, data1);
                            stream_handle.play_raw(samples);

                            //samples.pl

                            //sink.append(samples); 
                            
                            //sink.sleep_until_end();
         
                        }
                    }
                }
            }
        });
        }
    }

    fn window_event(
        &mut self,
        cx: &mut WindowContext,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.tx.as_ref().inspect(|tx| tx.send(PluginCommand::CloseWindow).unwrap());
            }

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
