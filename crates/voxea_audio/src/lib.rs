use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Devices, FromSample, Host, HostId, InputDevices, OutputDevices, Sample, SizedSample};
use log::{error, info};

// pub fn enumerate_hosts() -> Result<()> {
//     info!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
//     let available_hosts = cpal::available_hosts();
//     info!("Available hosts:\n  {:?}", available_hosts);
//
//     for host_id in available_hosts {
//         info!("{}", host_id.name());
//         let host = cpal::host_from_id(host_id)?;
//
//         let default_in = host.default_input_device().map(|e| e.name().unwrap());
//         let default_out = host.default_output_device().map(|e| e.name().unwrap());
//         info!("  Default Input Device:\n    {:?}", default_in);
//         info!("  Default Output Device:\n    {:?}", default_out);
//
//         let devices = host.devices()?;
//         info!("  Devices: ");
//         for (device_index, device) in devices.enumerate() {
//             info!("  {}. \"{}\"", device_index + 1, device.name()?);
//
//             // Input configs
//             if let Ok(conf) = device.default_input_config() {
//                 info!("    Default input stream config:\n      {:?}", conf);
//             }
//             let input_configs = match device.supported_input_configs() {
//                 Ok(f) => f.collect(),
//                 Err(e) => {
//                     info!("    Error getting supported input configs: {:?}", e);
//                     Vec::new()
//                 }
//             };
//             if !input_configs.is_empty() {
//                 info!("    All supported input stream configs:");
//                 for (config_index, config) in input_configs.into_iter().enumerate() {
//                     info!(
//                         "      {}.{}. {:?}",
//                         device_index + 1,
//                         config_index + 1,
//                         config
//                     );
//                 }
//             }
//
//             // Output configs
//             if let Ok(conf) = device.default_output_config() {
//                 info!("    Default output stream config:\n      {:?}", conf);
//             }
//             let output_configs = match device.supported_output_configs() {
//                 Ok(f) => f.collect(),
//                 Err(e) => {
//                     info!("    Error getting supported output configs: {:?}", e);
//                     Vec::new()
//                 }
//             };
//             if !output_configs.is_empty() {
//                 info!("    All supported output stream configs:");
//                 for (config_index, config) in output_configs.into_iter().enumerate() {
//                     info!(
//                         "      {}.{}. {:?}",
//                         device_index + 1,
//                         config_index + 1,
//                         config
//                     );
//                 }
//             }
//         }
//     }
//
//     Ok(())
// }

pub fn enumerate_hosts() -> Vec<HostId> {
    let available_hosts = cpal::available_hosts();

    available_hosts
    // available_hosts
    //     .into_iter()
    //     .filter_map(|host_id| cpal::host_from_id(host_id).ok())
    //     .collect()
}

pub fn enumerate_input_devices(id: &HostId) -> Vec<Device> {
    let host = cpal::host_from_id(*id).unwrap();

    host.input_devices().unwrap().collect()
}

pub fn enumerate_output_devices(id: &HostId) -> Vec<Device> {
    let host = cpal::host_from_id(*id).unwrap();

    host.output_devices().unwrap().collect()
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