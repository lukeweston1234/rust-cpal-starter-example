use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SampleFormat, SizedSample,
};
use i24::i24;
use itertools::{EitherOrBoth, Itertools};
use std::{
    env,
    sync::{Arc, Mutex},
};

fn load_wav(file_path: &str) -> Result<AudioSample, hound::Error> {
    println!("{:?}", env::current_dir());
    let reader = hound::WavReader::open(file_path)?;
    let spec = reader.spec();
    println!("WAV Spec: {:?}", spec);

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .collect::<Result<Vec<f32>, _>>()?,
        hound::SampleFormat::Int => reader
            .into_samples::<i32>()
            .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
            .collect::<Result<Vec<f32>, _>>()?,
    };

    Ok(AudioSample {
        samples,
        sample_rate: spec.sample_rate,
        position: 0,
    })
}

struct AudioSample {
    samples: Vec<f32>,
    sample_rate: u32,
    position: usize,
}

type SummedAudioHandle = Arc<Mutex<Option<AudioSample>>>;

fn main() {
    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("No output device");

    let stream_config = output_device
        .default_output_config()
        .expect("Could not get default output config");

    let sample_format = stream_config.sample_format();

    let drums = load_wav("assets/test_drums.wav").expect("Could not load drums!");
    let synth = load_wav("assets/test_synth.wav").expect("Could not load synth!");

    println!("Samples loaded!");

    let summed: Vec<f32> = drums
        .samples
        .into_iter()
        .zip_longest(synth.samples.iter())
        .map(|pair| match pair {
            EitherOrBoth::Both(a, b) => a + b,
            EitherOrBoth::Left(a) => a,
            EitherOrBoth::Right(b) => *b,
        })
        .collect();

    let summed_source = AudioSample {
        samples: summed,
        sample_rate: drums.sample_rate,
        position: 0,
    };

    let summed_handle = Arc::new(Mutex::new(Some(summed_source)));

    match sample_format {
        SampleFormat::F32 => run::<f32>(&output_device, &stream_config.into(), summed_handle),
        SampleFormat::I16 => run::<i16>(&output_device, &stream_config.into(), summed_handle),
        SampleFormat::U16 => run::<u16>(&output_device, &stream_config.into(), summed_handle),
        _ => panic!("Unsupported sample format!"),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, summed_handle: SummedAudioHandle)
where
    T: cpal::Sample + SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;

    let process = move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
        if let Ok(mut guard) = summed_handle.try_lock() {
            if let Some(audio_sample) = guard.as_mut() {
                for frame in data.chunks_mut(channels) {
                    for sample in frame.iter_mut() {
                        let s = audio_sample
                            .samples
                            .get(audio_sample.position)
                            .cloned()
                            .unwrap_or(0.0);

                        // Convert the f32 sample to the output sample type T
                        *sample = cpal::Sample::from_sample(s * 5.0);

                        audio_sample.position += 1;
                    }
                }
            }
        }
    };

    let stream = device
        .build_output_stream(
            config,
            process,
            |err| eprintln!("An error occurred in the output stream: {}", err),
            None,
        )
        .expect("Could not build output stream");

    stream.play().expect("Could not play stream!");

    std::thread::sleep(std::time::Duration::from_secs(10));
}
