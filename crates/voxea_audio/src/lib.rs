use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, HostId, Sample, SizedSample, StreamConfig};
use log::{error, info, warn};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use rustc_hash::FxHashMap;
use std::cell::UnsafeCell;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex, RwLock};
use voxea_alloc::perf::{self, PerfTrace};
use voxea_vst::base::funknown::IAudioProcessor_Impl;
use voxea_vst::vst::audio_processor::{
    AudioBusBuffers, ProcessContext, ProcessData, ProcessMode, SymbolicSampleSize,
};
use vst::host::{HostParameterChanges, VSTHostContext};

pub mod vst;

pub fn enumerate_hosts() -> Vec<HostId> {
    let available_hosts = cpal::available_hosts();

    available_hosts
}

pub fn enumerate_input_devices(id: &HostId) -> Vec<Device> {
    let host = cpal::host_from_id(*id).unwrap();

    host.input_devices().unwrap().collect()
}

pub fn enumerate_output_devices(id: &HostId) -> Vec<Device> {
    let host = cpal::host_from_id(*id).unwrap();

    host.output_devices().unwrap().collect()
}

pub fn default_input_device(id: &HostId) -> Device {
    let host = cpal::host_from_id(*id).unwrap();

    host.default_input_device().unwrap()
}

pub fn default_output_device(id: &HostId) -> Device {
    let host = cpal::host_from_id(*id).unwrap();

    host.default_output_device().unwrap()
}

pub fn beep() -> Result<()> {
    let host = cpal::default_host();

    let opt = "default";

    let device = if opt == "default" {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == opt).unwrap_or(false))
    }
    .expect("failed to find output device");
    info!("Output device: {}", device.name()?);

    let config = device.default_output_config()?;
    info!("Default output config: {:?}", config);

    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        // cpal::SampleFormat::I24 => run::<I24>(&device, &config.into()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into()),
        // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into()),
        // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into()),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<()>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| error!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value);
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

pub fn record() -> Result<()> {
    let host = cpal::default_host();

    let opt = "default";

    // Set up the input device and stream with the default input config.
    let device = if opt == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == opt).unwrap_or(false))
    }
    .expect("failed to find input device");

    info!("Input device: {}", device.name()?);

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    info!("Default input config: {:?}", config);

    // The WAV file we're recording to.
    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    let spec = wav_spec_from_config(&config);
    let writer = hound::WavWriter::create(PATH, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    info!("Begin recording...");

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();

    let err_fn = move |err| {
        error!("an error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            err_fn,
            None,
        )?,
        sample_format => {
            return Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };

    stream.play()?;

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize()?;
    info!("Recording {} complete!", PATH);
    Ok(())
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

fn decode_wav_file(file: String) -> Result<()> {
    let wav = hound::WavReader::open(file)?;

    Ok(())
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

#[repr(C)]
#[derive(Clone)]
pub struct Sync2DArray<T: Copy + Clone + Sized, const CHANNELS: usize, const BUFFER_SIZE: usize> {
    references: Arc<[*mut T; CHANNELS]>,
    data: Arc<UnsafeCell<[[T; BUFFER_SIZE]; CHANNELS]>>,
    buffer_size: usize,
}

unsafe impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize> Sync
    for Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
}
unsafe impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize> Send
    for Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
}

impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize>
    Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
    fn new(default: T, buffer_size: usize) -> Self {
        unsafe {
            let data = Arc::new(UnsafeCell::new([[default; BUFFER_SIZE]; CHANNELS]));
            let references = Arc::new(std::array::from_fn(|i| (*data.get())[i].as_mut_ptr()));
            warn!("{:?} references!", references.len());
            Self {
                references,
                data,
                buffer_size,
            }
        }
    }

    pub fn read(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data.get() as *mut _, self.buffer_size * CHANNELS)
        }
    }

    pub fn as_ptr(&mut self) -> *const *mut T {
        self.references.as_ptr()
    }

    pub fn write(&mut self, channel: usize, idx: usize, sample: T) {
        unsafe {
            (*self.data.get())[channel][idx] = sample;
        }
    }

    pub fn as_ref(&self) -> &[&mut [T; BUFFER_SIZE]; CHANNELS] {
        unsafe {
            &*std::mem::transmute::<*const [*mut T; CHANNELS], *mut [&mut [T; BUFFER_SIZE]; CHANNELS]>(
                Arc::into_raw(self.references.clone()),
            )
        }
    }

    pub fn as_mut_ref(&mut self) -> &mut [&mut [T; BUFFER_SIZE]; CHANNELS] {
        unsafe {
            &mut *std::mem::transmute::<
                *const [*mut T; CHANNELS],
                *mut [&mut [T; BUFFER_SIZE]; CHANNELS],
            >(Arc::into_raw(self.references.clone()))
        }
    }
}

impl<T: Copy + Clone, const CHANNELS: usize, const BUFFER_SIZE: usize> Drop
    for Sync2DArray<T, CHANNELS, BUFFER_SIZE>
{
    fn drop(&mut self) {
        warn!("Dropping Sync2DArray!");
    }
}

const MAX_BLOCK_SIZE: usize = 2048;

pub struct AudioEngine {
    // Currently selected host
    pub host: cpal::Host,

    // Currently selected input and output devices
    pub input_device: cpal::Device,
    pub output_device: cpal::Device,

    pub input_config: cpal::StreamConfig,
    pub output_config: cpal::StreamConfig,

    pub input_stream: Option<cpal::Stream>,
    pub output_stream: Option<cpal::Stream>,

    // Data needed to pass through FFI stored as global memory on the heap for optimization
    pub input_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,
    pub output_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,
    pub resampled_data: Sync2DArray<f32, 2, MAX_BLOCK_SIZE>,

    pub in_bus: Arc<UnsafeCell<AudioBusBuffers>>,
    pub out_bus: Arc<UnsafeCell<AudioBusBuffers>>,

    pub input_params: Arc<UnsafeCell<HostParameterChanges>>,
    pub process_context: Arc<UnsafeCell<ProcessContext>>,
    pub process_data: Arc<ProcessData>,

    pub plugin_modules: Arc<RwLock<Vec<VSTHostContext>>>,

    // Cache input and output devices on startup since for certain drivers like ASIO, selecting a
    // device removes all other devices.
    pub cached_input_devices: FxHashMap<HostId, Vec<String>>,
    pub cached_output_devices: FxHashMap<HostId, Vec<String>>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        // https://github.com/RustAudio/cpal/issues/884
        // https://github.com/RustAudio/cpal/issues/657

        let mut cached_input_devices = FxHashMap::default();
        let mut cached_output_devices = FxHashMap::default();

        cpal::available_hosts().iter().for_each(|id| {
            let Ok(host) = cpal::host_from_id(*id) else {
                warn!("Failed to get host from id: {:?}", id.name());
                return;
            };

            let Ok(input_devices) = host.input_devices() else {
                warn!("Failed to get input devices for host {:?}!", id.name());
                return;
            };

            let input_devices = input_devices
                .map(|f| f.name().unwrap().to_string())
                .collect::<Vec<String>>();

            let Ok(output_devices) = host.output_devices() else {
                warn!("Failed to get output devices for host {:?}!", id.name());
                return;
            };

            let output_devices = output_devices
                .map(|f| f.name().unwrap().to_string())
                .collect::<Vec<String>>();

            cached_input_devices
                .entry(id.clone())
                .and_modify(|f: &mut Vec<String>| f.extend(input_devices.clone()))
                .or_insert(input_devices.clone());
            cached_output_devices
                .entry(id.clone())
                .and_modify(|f: &mut Vec<String>| f.extend(output_devices.clone()))
                .or_insert(output_devices.clone());
        });

        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .expect("Failed to get default input device!");
        let output_device = host
            .default_output_device()
            .expect("Failed to get defaut output device!");

        info!(
            "Supported Input Configs: {:?}",
            input_device
                .supported_input_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );
        info!(
            "Supported Output Configs: {:?}",
            output_device
                .supported_output_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );

        let input_config = input_device.default_input_config().unwrap().into();
        let output_config = output_device.default_output_config().unwrap().into();

        info!("Creating AudioEngine with:\n\tHost: {:?}\n\tInput: {:?}\n\tOutput: {:?}\n\tConfig: {:?}", host.id(), input_device.name(), output_device.name(), input_config);
        // info!("\tHost: {:?}", host.id());
        // info!("\tInput: {:?}", input_device.name());
        // info!("\tOutput: {:?}", output_device.name());
        // info!("\tConfig: {:?}", config);
        //

        let mut input_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);
        let mut output_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);
        let resampled_data = Sync2DArray::<f32, 2, MAX_BLOCK_SIZE>::new(0.0f32, MAX_BLOCK_SIZE);

        let in_bus = Arc::new(UnsafeCell::new(AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0,
            channel_buffers_32: input_data.as_ptr() as *mut _,
        }));

        let out_bus = Arc::new(UnsafeCell::new(AudioBusBuffers {
            num_channels: 2,
            silence_flags: 0,
            channel_buffers_32: output_data.as_ptr() as *mut _,
        }));

        let input_params = Arc::new(UnsafeCell::new(HostParameterChanges::new()));
        let process_context = Arc::new(UnsafeCell::new(ProcessContext { padding: [0; 200] }));

        let process_data = Arc::new(ProcessData {
            process_mode: ProcessMode::Realtime,
            symbolic_sample_size: SymbolicSampleSize::Sample32,
            num_samples: 192 as i32,
            num_inputs: 1,
            num_outputs: 1,
            inputs: in_bus.get(),
            outputs: out_bus.get(),
            input_parameter_changes: input_params.get() as *mut _,
            output_parameter_changes: std::ptr::null_mut(),
            input_events: std::ptr::null_mut(),
            output_events: std::ptr::null_mut(),
            process_context: std::ptr::null_mut(),
        });

        let plugin_modules = Arc::new(RwLock::new(Vec::new()));

        Self {
            host,
            input_device,
            output_device,
            input_config,
            output_config,

            input_stream: None,
            output_stream: None,

            input_data,
            output_data,
            resampled_data,

            in_bus,
            out_bus,

            input_params,
            process_context,

            process_data,

            plugin_modules,

            cached_input_devices,
            cached_output_devices,
        }
    }
}

impl AudioEngine {
    pub fn run(&mut self) {
        let channels = self.input_config.channels as usize;
        let plugin_modules = self.plugin_modules.clone();
        let buffer_size = 192;

        let ring = HeapRb::<f32>::new(buffer_size * channels * 2);
        let (mut producer, mut consumer) = ring.split();
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        let mut resampler = SincFixedIn::<f32>::new(
            self.output_config.sample_rate.0 as f64 / self.input_config.sample_rate.0 as f64,
            2.0,
            params,
            buffer_size,
            channels,
        )
        .unwrap();

        let process_data = self.process_data.clone();
        let mut input_data = self.input_data.clone();
        let output_data = self.output_data.clone();
        let mut resampled_data = self.resampled_data.clone();

        let input_stream = self
            .input_device
            .build_input_stream(
                &self.input_config,
                move |data: &[i32], _: &cpal::InputCallbackInfo| {
                    // perf::begin_perf!("audio_input_stream");
                    let block_size = data.len() / channels;

                    for (i, frame) in data.chunks(channels).enumerate() {
                        for j in 0..channels {
                            input_data.write(j, i, frame[j] as f32 / i32::MAX as f32);
                        }
                    }

                    unsafe {
                        for plugin in plugin_modules.read().unwrap().iter() {
                            let data = process_data.clone();
                            plugin
                                .processor
                                .as_ref()
                                .unwrap()
                                .process(Arc::into_raw(data) as *mut _);
                        }
                    }

                    resampler
                        .process_partial_into_buffer(
                            Some(output_data.as_ref()),
                            resampled_data.as_mut_ref(),
                            None,
                        )
                        .unwrap();

                    for i in 0..block_size {
                        resampled_data.as_ref().iter().for_each(|v| {
                            let Some(sample) = v.get(i) else {
                                return;
                            };

                            let _ = producer.try_push(*sample);
                        });
                    }
                },
                |err| {
                    error!("Input stream error! {:?}", err);
                },
                None,
            )
            .expect("Faied to create input stream!");

        let output_stream = self
            .output_device
            .build_output_stream(
                &self.output_config,
                move |data: &mut [i32], _: &cpal::OutputCallbackInfo| {
                    for sample in data {
                        *sample = match consumer.try_pop() {
                            Some(s) => {
                                let scaled = s * i32::MAX as f32;
                                // Clamp the value to ensure it stays within the valid i32 range
                                scaled.round().clamp(i32::MIN as f32, i32::MAX as f32) as i32
                            }
                            None => 0,
                        };
                    }
                },
                |err| {
                    error!("Output stream error! {:?}", err);
                },
                None,
            )
            .expect("Faied to create output stream!");

        input_stream.play().unwrap();
        output_stream.play().unwrap();

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);
    }

    pub fn add_plugin(&mut self, path: &str) {
        let Ok(mut plugin) = VSTHostContext::new(path) else {
            warn!("Failed to load plugin: {:?}", path);
            return;
        };

        unsafe {
            plugin.processor.as_mut().unwrap().set_processing(true);
        }

        self.plugin_modules.write().unwrap().push(plugin);
    }

    pub fn select_host(&mut self, host: &str) {
        self.input_stream.take().unwrap().pause().unwrap();
        self.output_stream.take().unwrap().pause().unwrap();

        let Some(host) = cpal::available_hosts()
            .into_iter()
            .find(|id| id.name() == host)
            .map_or(None, |id| cpal::host_from_id(id).ok())
        else {
            warn!("Failed to get host: {:?}", host);
            return;
        };

        warn!(
            "Available Input Devices: {:?}",
            host.input_devices()
                .unwrap()
                .map(|f| f.name().unwrap())
                .collect::<Vec<_>>()
        );

        let input_device = host
            .default_input_device()
            .expect("Failed to get default input device!");

        // https://stackoverflow.com/questions/78319116/no-audio-input-via-asio-with-feedback-example-using-cpal
        // Since ASIO expects input/output to be exclusive, they need to be the same device.
        let output_device = if host.id() == cpal::HostId::Asio {
            input_device.clone()
        } else {
            host.default_output_device()
                .expect("Failed to get defaut output device!")
        };

        info!(
            "Supported Input Configs: {:?}",
            input_device
                .supported_input_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );
        info!(
            "Supported Output Configs: {:?}",
            output_device
                .supported_output_configs()
                .unwrap()
                .collect::<Vec<_>>()
        );

        let input_config = input_device.default_input_config().unwrap().into();
        let output_config = output_device.default_output_config().unwrap().into();

        info!("Creating AudioEngine with:\n\tHost: {:?}\n\tInput: {:?}\n\tOutput: {:?}\n\tConfig: {:?}", host.id(), input_device.name(), output_device.name(), input_config);

        self.host = host;

        self.input_device = input_device;
        self.output_device = output_device;

        self.input_config = input_config;
        self.output_config = output_config;

        self.run();
    }

    pub fn select_input_device(&mut self, device_name: String) {
        self.input_stream.take().unwrap().pause().unwrap();
        self.output_stream.take().unwrap().pause().unwrap();

        // TODO(@Aliremu): Temporary fix to make sure ASIO devices are refreshed when trying to
        // choose another device.
        let old_host = self.host.id();
        self.host = cpal::default_host();
        self.input_device = self.host.default_input_device().unwrap();
        self.input_config = self.input_device.default_input_config().unwrap().into();

        self.host = cpal::host_from_id(old_host).unwrap();

        let input_device = self
            .host
            .input_devices()
            .unwrap()
            .find(|device| device.name().unwrap() == device_name)
            .unwrap();

        let input_config: StreamConfig = input_device.default_input_config().unwrap().into();

        info!(
            "Supported Input Configs: {:?}\nChosen Input Config: {:?}",
            input_device
                .supported_input_configs()
                .unwrap()
                .collect::<Vec<_>>(),
            input_config
        );

        if self.host.id() == cpal::HostId::Asio {
            let output_device = input_device.clone();

            self.output_device = output_device;
            self.output_config = input_config.clone();
        }

        self.input_device = input_device;
        self.input_config = input_config;

        self.run();
    }

    pub fn select_output_device(&mut self, device_name: String) {
        self.input_stream.take().unwrap().pause().unwrap();
        self.output_stream.take().unwrap().pause().unwrap();

        let old_host = self.host.id();
        self.host = cpal::default_host();
        self.output_device = self.host.default_output_device().unwrap();
        self.output_config = self.output_device.default_output_config().unwrap().into();

        self.host = cpal::host_from_id(old_host).unwrap();

        let output_device = self
            .host
            .output_devices()
            .unwrap()
            .find(|device| device.name().unwrap() == device_name)
            .unwrap();

        let output_config: StreamConfig = output_device.default_output_config().unwrap().into();

        info!(
            "Supported Output Configs: {:?}\nChosen Output Config: {:?}",
            output_device
                .supported_output_configs()
                .unwrap()
                .collect::<Vec<_>>(),
            output_config
        );

        if self.host.id() == cpal::HostId::Asio {
            let input_device = output_device.clone();

            self.input_device = input_device;
            self.input_config = output_config.clone();
        }

        self.output_device = output_device;
        self.output_config = output_config;

        self.run();
    }
    pub fn enumerate_input_devices(&self) -> Vec<Device> {
        self.host
            .input_devices()
            .expect(&format!(
                "Failed to get devices for host: {:?}",
                self.host.id().name()
            ))
            .collect()
    }

    pub fn enumerate_output_devices(&self) -> Vec<Device> {
        self.host
            .output_devices()
            .expect(&format!(
                "Failed to get devices for host: {:?}",
                self.host.id().name()
            ))
            .collect()
    }
}
