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
    data: Arc<[[f32; M]; N]>,
}

unsafe impl<const N: usize, const M: usize> Sync for Sync2DArray<N, M> {}
unsafe impl<const N: usize, const M: usize> Send for Sync2DArray<N, M> {}

impl<const N: usize, const M: usize> Sync2DArray<N, M> {
    pub fn new() -> Arc<Self> {
        let data = Arc::new([[0.0f32; M]; N]);

        Arc::new(Sync2DArray {
            channel_l: data[0].as_ptr() as *mut _,
            channel_r: data[1].as_ptr() as *mut _,
            data,
        })
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
                const BLOCK_SIZE: usize = 480;
                const FREQUENCY: f32 = 440.0;
                let phase_step = 2.0 * std::f32::consts::PI * FREQUENCY / SAMPLE_RATE;
                let phase: f32 = 0.0;

                let start = Instant::now();
                let frame_time = Duration::from_secs_f64(BLOCK_SIZE as f64 / SAMPLE_RATE as f64);

                let mut left_in = [0.5f32; BLOCK_SIZE];
                let mut right_in = [0.5f32; BLOCK_SIZE];
                let mut data1 = [&mut left_in, &mut right_in];

                let mut left_out = [0.0f32; BLOCK_SIZE];
                let mut right_out = [0.0f32; BLOCK_SIZE];
                let mut data2 = [&mut left_out, &mut right_out];

                // let host = cpal::available_hosts().iter().find(|id| id.name() == "ASIO").map(|id| cpal::host_from_id(*id)).unwrap().unwrap();
                let host = cpal::default_host();
                warn!("Chosen host: {:?}", host.id().name());
                warn!(
                    "HOSTS: {:?}",
                    host.input_devices()
                        .unwrap()
                        .map(|device| device.name())
                        .collect::<Vec<_>>()
                );
                // let output_device = host.output_devices().unwrap().find(|device| device.name().unwrap() == "Focusrite USB ASIO").unwrap();
                let output_device = host.default_output_device().unwrap();
                let input_device = host.default_input_device().unwrap();
                // let input_device = host
                // .input_devices()
                // .unwrap()
                // .find(|device| device.name().unwrap() == "Analogue 1 + 2 (Focusrite USB Audio)")
                // .unwrap();
                // let config: cpal::StreamConfig = output_device.default_output_config().unwrap().into();
                let config = cpal::StreamConfig {
                    channels: 2,
                    sample_rate: cpal::SampleRate(SAMPLE_RATE as u32),
                    buffer_size: cpal::BufferSize::Fixed(BLOCK_SIZE.try_into().unwrap()),
                };
                // let config = cpal::SupportedStreamConfig::new(2, cpal::SampleRate(44100), cpal::SupportedBufferSize::Unknown, cpal::SampleFormat::F32).into();
                let input_config = input_device.default_input_config().unwrap().into();
                let output_config = output_device.default_output_config().unwrap().into();

                warn!("Input Device: {:?}", input_device.name());
                warn!("Output Device: {:?}", output_device.name());
                warn!(
                    "Configs: {:?}\nConfig: {:?}",
                    output_device
                        .supported_output_configs()
                        .unwrap()
                        .collect::<Vec<_>>(),
                    config
                );
                // let latency = 5.0;
                // let latency_frames = (latency / 1_000.0) * SAMPLE_RATE as f32;
                let latency_samples = BLOCK_SIZE * 2; //latency_frames as usize * 2.0 as usize;

                // The buffer to share samples
                let ring = HeapRb::<f32>::new(latency_samples * 2);
                let (mut producer, mut consumer) = ring.split();
                // Fill the samples with 0.0 equal to the length of the delay.
                for _ in 0..latency_samples {
                    // The ring buffer has twice as much space as necessary to add latency here,
                    // so this should never fail
                    producer.try_push(0.0).unwrap();
                }

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
                let output_params: *mut c_void = std::ptr::null_mut();

                let mut process_data = ProcessData {
                    process_mode: ProcessMode::Realtime,
                    symbolic_sample_size: SymbolicSampleSize::Sample32,
                    num_samples: BLOCK_SIZE as i32,
                    num_inputs: 1,
                    num_outputs: 1,
                    inputs: &mut in_bus,
                    outputs: &mut out_bus,
                    input_parameter_changes: &mut input_params as *mut _ as *mut c_void,
                    output_parameter_changes: std::ptr::null_mut(),
                    input_events: std::ptr::null_mut(),
                    output_events: std::ptr::null_mut(),
                    process_context: std::ptr::null_mut(),
                };

                processor.set_processing(true);

                let output_stream = output_device
                    .build_output_stream(
                        &output_config,
                        move |data: &mut [i32], info: &cpal::OutputCallbackInfo| {
                            // warn!("Output data len: {:?}", info);
                            // for frame in data {
                            // consumer.try_pop();

                            // let sample = (phase).sin();
                            // phase += phase_step;

                            // Wrap phase to prevent overflow
                            // if phase >= 2.0 * std::f32::consts::PI {
                            // phase -= 2.0 * std::f32::consts::PI;
                            // }

                            // Assign sample to both left and right channels (stereo)
                            // *frame = (sample * i32::MAX as f32).round().clamp(i32::MIN as f32, i32::MAX as f32) as i32; // Left channel
                            // frame[3] = sample; // Right channel
                            // }

                            for sample in data {
                                *sample = match consumer.try_pop() {
                                    Some(s) => {
                                        let scaled = s * i32::MAX as f32;
                                        // Clamp the value to ensure it stays within the valid i32 range
                                        scaled.round().clamp(i32::MIN as f32, i32::MAX as f32)
                                            as i32
                                    }
                                    None => 0,
                                };
                            }

                            // for sample in data.chunks_mut(2) {
                            // let l = match consumer.try_pop() {
                            // Some(s) => {
                            // let scaled = s * i32::MAX as f32;
                            // Clamp the value to ensure it stays within the valid i32 range
                            // scaled.round().clamp(i32::MIN as f32, i32::MAX as f32) as i32
                            // }

                            // None => {
                            // 0
                            // }
                            // };

                            // let r = match consumer.try_pop() {
                            // Some(s) => {
                            // let scaled = s * i32::MAX as f32;
                            // Clamp the value to ensure it stays within the valid i32 range
                            // scaled.round().clamp(i32::MIN as f32, i32::MAX as f32) as i32
                            // }

                            // None => {
                            // 0
                            // }
                            // };

                            // sample[0] = l;
                            // sample[1] = r;
                            // }
                        },
                        move |err| {
                            warn!("STREAM ERROR: {:?}", err);
                        },
                        None,
                    )
                    .unwrap();

                let process_data_ptr = &mut process_data as *mut _ as usize;
                let data1_ptr = data1.as_mut_ptr() as usize;
                let data2_ptr = data2.as_mut_ptr() as usize;

                let input_stream = input_device
                    .build_input_stream(
                        &input_config,
                        move |data: &[i32], _: &cpal::InputCallbackInfo| {
                            // warn!("Input data len: {:?}", data.len());
                            let raw_data1: &mut [&mut [f32; BLOCK_SIZE]] =
                                std::slice::from_raw_parts_mut(
                                    data1_ptr as *mut &mut [f32; BLOCK_SIZE],
                                    2,
                                );
                            let raw_data2: &mut [&mut [f32; BLOCK_SIZE]] =
                                std::slice::from_raw_parts_mut(
                                    data2_ptr as *mut &mut [f32; BLOCK_SIZE],
                                    2,
                                );
                            for (i, frame) in data.chunks(2).enumerate() {
                                raw_data1[0][i] = frame[0] as f32 / i32::MAX as f32;
                                raw_data1[1][i] = frame[1] as f32 / i32::MAX as f32;
                            }

                            let raw_process_data =
                                std::mem::transmute::<usize, &mut ProcessData>(process_data_ptr);
                            processor.process(raw_process_data);

                            let mut interleaved = [0.0; BLOCK_SIZE * 2];

                            for (i, frame) in interleaved.chunks_mut(2).enumerate() {
                                // frame[0] = raw_data2[0][i];
                                // frame[1] = raw_data2[1][i];
                                producer.try_push(raw_data2[0][i]);
                                producer.try_push(raw_data2[1][i]);
                            }
                        },
                        move |err| {},
                        None,
                    )
                    .unwrap();

                input_stream.play().unwrap();
                output_stream.play().unwrap();

                loop {
                    match rx.try_recv() {
                        Ok(_msg) => {
                            warn!("Window closed!");
                            drop(input_stream);
                            drop(output_stream);
                            processor.set_processing(false);
                            panic!();
                        }

                        Err(_err) => {
                            // if Instant::now() > start {
                            // start = start + frame_time;
                            // Generate a sine wave for BLOCK_SIZE samples
                            // for n in 0..BLOCK_SIZE {
                            // let sample = (phase).sin();
                            // phase += phase_step;

                            // Wrap phase around to prevent overflow
                            // if phase >= 2.0 * std::f32::consts::PI {
                            // phase -= 2.0 * std::f32::consts::PI;
                            // }

                            // Assign sample to both channels (stereo)
                            // data1[0][n] = sample;
                            // data1[1][n] = sample;
                            // }

                            // processor.process(&mut process_data);

                            // producer.push_slice(data2[0]);

                            // }

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
                self.tx.as_ref().inspect(|tx| {
                    let _ = tx.send(PluginCommand::CloseWindow);
                });
            }

            WindowEvent::Resized(size) => {
                unsafe {
                    let mut view = self.vst.view.unwrap();
                    let mut rect = ViewRect::default();
                    view.check_size_constraint(&mut rect);
                    cx.window
                        .window
                        .request_inner_size(PhysicalSize::new(rect.right, rect.bottom));
                    warn!("SIZE CONSTRAINTS: {:?}", rect);
                    // (*(view)).on_size(Box::into_raw(rect) as *const c_void);
                }
            }

            _ => {}
        };
    }

    fn render(&mut self, cx: &mut WindowContext, event_loop: &ActiveEventLoop) {}
}
