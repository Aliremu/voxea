use crate::vst::host::{HostParameterChanges, VSTHostContext};
use crate::window::{Render, WindowContext};
use crate::App;
use cpal::traits::{HostTrait, StreamTrait};
use log::warn;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rodio::DeviceTrait;
use voxea_vst::vst::audio_processor::{AudioBusBuffers, IParameterChanges, ProcessData, ProcessMode, SymbolicSampleSize};
use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use voxea_vst::base::funknown::{IAudioProcessor_Impl, IPlugView_Impl};
use voxea_vst::gui::plug_view::{PlatformType, ViewRect};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::WindowAttributes;

pub fn init(cx: &mut App, event_loop: &ActiveEventLoop, plug: u32) {
    let window_attributes = WindowAttributes::default()
        .with_title("Plugin")
        .with_inner_size(PhysicalSize::new(1200, 800));

    let path = match plug {
        1 => "C:/Coding/RustRover/voxea/vst3/Archetype Nolly.vst3",
        2 => "C:/Coding/RustRover/voxea/vst3/LABS.vst3",
        3 => "C:/Coding/RustRover/voxea/vst3/ZamDelay.vst3",

        _ => unimplemented!(),
    };
    let vst = VSTHostContext::new(path).unwrap();

    let plugin = PlugView {
        vst: Arc::new(vst),
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
    vst: Arc<VSTHostContext>,
    tx: Option<mpsc::Sender<PluginCommand>>,
}

#[repr(C)]
pub struct Sync2DArray<const N: usize, const M: usize> {
    channel_l: *mut (),
    channel_r: *mut (),
    data: Arc<[[f32; M]; N]>
}

unsafe impl<const N: usize, const M: usize> Sync for Sync2DArray<N, M> {}
unsafe impl<const N: usize, const M: usize> Send for Sync2DArray<N, M> {}

impl<const N: usize, const M: usize> Sync2DArray<N, M> {
    pub fn new() -> Arc<Self> {
        let data = Arc::new([[0.0f32; M]; N]);
        
        Arc::new(Sync2DArray { 
            channel_l: data[0].as_ptr() as *mut _,
            channel_r: data[1].as_ptr() as *mut _,
            data
        })
    }
}

pub struct TestDrop {
}

impl Drop for TestDrop {
    fn drop(&mut self) {
        // warn!("TESTDROP HAS BEEN DROPPED???????????");
    }
}

impl Render for PlugView {
    fn on_open(&mut self, cx: &mut WindowContext) {
        let window_handle = cx.window.window.window_handle().unwrap().as_raw();

        let hwnd = match window_handle {
            RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => todo!("Not running on Windows"),
        };

        unsafe {
            self.vst
                .view
                .unwrap()
                .attached(hwnd as *mut c_void, PlatformType::HWND);
        }

        let vst = self.vst.clone();
        let (tx, rx) = mpsc::channel();

        self.tx = Some(tx);
        
        // #[cfg(test ="")]
        std::thread::spawn(move || {
            unsafe {
                let mut processor = vst.processor.unwrap();

                const SAMPLE_RATE: f32 = 48000.0;
                const BLOCK_SIZE: usize = 960;
                const FREQUENCY: f32 = 440.0;
                let phase_step = 2.0 * std::f32::consts::PI * FREQUENCY / SAMPLE_RATE;
                let mut phase: f32 = 0.0;

                let mut start = Instant::now();

                let mut left_in = [0.5f32; BLOCK_SIZE];
                let mut right_in = [0.5f32; BLOCK_SIZE];
                let mut data1 = [&mut left_in, &mut right_in];


                let mut left_out = [0.0f32; BLOCK_SIZE];
                let mut right_out = [0.0f32; BLOCK_SIZE];
                let mut data2 = [&mut left_out, &mut right_out];

                let host = cpal::default_host();
                let device = host.default_output_device().unwrap();
                let input_device = host.default_input_device().unwrap();
                let config: cpal::StreamConfig = device.default_output_config().unwrap().into();
                // let config = cpal::StreamConfig {
                    // channels: 2,
                    // sample_rate: cpal::SampleRate(44100),
                    // buffer_size: cpal::BufferSize::Default
                // };
                warn!("Config: {:?}", config);
                // cursed
                let raw_ptr = data2.as_mut_ptr() as usize;
                let latency = 5.0;
                let latency_frames = (latency / 1_000.0) * SAMPLE_RATE as f32;
                let latency_samples = 4410 * 2; //latency_frames as usize * 2.0 as usize;

                
                // The buffer to share samples
                let ring = HeapRb::<f32>::new(latency_samples * 2);
                let (mut producer, mut consumer) = ring.split();
                // Fill the samples with 0.0 equal to the length of the delay.
                for _ in 0..latency_samples {
                    // The ring buffer has twice as much space as necessary to add latency here,
                    // so this should never fail
                    producer.try_push(0.0).unwrap();
                }
                let stream = device.build_output_stream(
                    &config, 
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        // let raw: &mut [&mut [f32; BLOCK_SIZE]] = std::slice::from_raw_parts_mut(raw_ptr as *mut &mut [f32; BLOCK_SIZE], 2);
                        // for (output_sample, &input_sample) in data.iter_mut().zip(raw[0].iter()) {
                            // *output_sample = input_sample;
                        // }
                        //
                        // for frame in data.chunks_mut(2) {
                            // let sample = (phase).sin();
                            // phase += phase_step;
                            
                            // Wrap phase to prevent overflow
                            // if phase >= 2.0 * std::f32::consts::PI {
                                // phase -= 2.0 * std::f32::consts::PI;
                            // }

                            // Assign sample to both left and right channels (stereo)
                            // frame[0] = sample; // Left channel
                            // frame[3] = sample; // Right channel
                        // }
                        //

                        
                            // for (i, frame) in data.chunks_mut(2).enumerate() {
                                // if i >= BLOCK_SIZE {
                                    // break;
                                // }

                                // frame[0] = raw[0][i % BLOCK_SIZE];  // Left channel
                                // frame[1] = raw[1][i % BLOCK_SIZE]; // Right channel
                            // }
                            //
                            //
                        for sample in data.chunks_mut(2) {
                            let s = match consumer.try_pop() {
                                Some(s) => s,
                                None => {
                                    0.0
                                }
                            };
                            
                            // *sample = s;
                            sample[0] = s;
                            sample[1] = s;
                        }

                    }, 
                    move |err| {
                        warn!("STREAM ERROR: {:?}", err);
                    },
                    None).unwrap();

                // let record = input_device.build_input_stream(
                    // &config, 
                    // move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // for &sample in data {
                            // if producer.try_push(sample).is_err() {
                            // 
                            // }
                        // }
                    // }, 
                    // move |err| {
                    // }, 
                    // None).unwrap();

                // record.play().unwrap();
                stream.play().unwrap();

                let mut in_bus = AudioBusBuffers {
                    num_channels: 2,
                    silence_flags: 0,
                    channel_buffers_32: data1.as_mut_ptr() as *mut _,
                };

                let mut out_bus = AudioBusBuffers {
                    num_channels: 2,
                    silence_flags: 0,
                    channel_buffers_32: data2.as_mut_ptr() as *mut _,
                };

                let mut input_params = HostParameterChanges::new();
                let mut output_params: *mut c_void = std::ptr::null_mut();

                let mut data = ProcessData {
                    process_mode: ProcessMode::Realtime,
                    symbolic_sample_size: SymbolicSampleSize::Sample32,
                    num_samples: BLOCK_SIZE as i32,
                    num_inputs: 1,
                    num_outputs: 1,
                    inputs: &mut in_bus,
                    outputs: &mut out_bus,
                    input_parameter_changes: &mut input_params as *mut _ as *mut c_void,
                    output_parameter_changes: None,
                    input_events: None,
                    output_events: None,
                    process_context: None,
                };

                processor.set_processing(true);

                loop {
                    match rx.try_recv() {
                        Ok(_msg) => {
                            warn!("Window closed!");
                            processor.set_processing(false);
                            panic!();
                        }

                        Err(_err) => {
                            if(start.elapsed() > Duration::from_millis(19)) {
                            // Generate a sine wave for BLOCK_SIZE samples
                            for n in 0..BLOCK_SIZE {
                                let sample = (phase).sin();
                                phase += phase_step;

                                // Wrap phase around to prevent overflow
                                if phase >= 2.0 * std::f32::consts::PI {
                                    phase -= 2.0 * std::f32::consts::PI;
                                }

                                // Assign sample to both channels (stereo)
                                data1[0][n] = sample;
                                data1[1][n] = sample;
                            }

                            processor.process(&mut data);

                            producer.push_slice(data2[0]);
                            // producer.push_slice(data2[1]);
                            start = Instant::now();
                            }

                            // for (l, r) in data2[0].iter().zip(data2[1].iter()) {
                                // producer.try_push(*l);
                                // producer.try_push(*r);
                            // }
                            // warn!("{:?}", data2);
                        }
                    }
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
            WindowEvent::CloseRequested => {
                self.tx
                    .as_ref()
                    .inspect(|tx| {
                        let _ = tx.send(PluginCommand::CloseWindow);
                    });
            }

            WindowEvent::Resized(size) => {
                unsafe {
                    let mut view = self.vst.view.unwrap();
                    let mut rect = ViewRect::default();
                    view.check_size_constraint(&mut rect);
                    cx.window.window.request_inner_size(PhysicalSize::new(rect.right, rect.bottom));
                    warn!("SIZE CONSTRAINTS: {:?}", rect);
                    // (*(view)).on_size(Box::into_raw(rect) as *const c_void);
                }
            }

            _ => {}
        };
    }

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {}
}
