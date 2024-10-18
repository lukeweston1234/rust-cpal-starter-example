use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SampleFormat, SizedSample,
};
use ringbuf::{
    storage::Heap,
    traits::{Consumer, Producer, Split},
    wrap::caching::Caching,
    HeapRb, SharedRb,
};
use std::{
    env, error,
    sync::{Arc, Mutex},
};
use stream::{get_input_stream, get_output_stream, RingBufConsumer};

mod stream;

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
    let drums = load_wav("assets/drums_32.wav").expect("Could not load drums!");
    let synth = load_wav("assets/synth_32.wav").expect("Could not load synth!");

    println!("Samples loaded!");

    fn sum_sample_array_items(samples: Vec<Vec<f32>>) -> Vec<f32> {
        let max_length = samples.iter().map(|x| x.len()).max().unwrap_or(0);
        let mut empty_buffer = vec![0.0; max_length];
        for i in 0..max_length - 1 {
            for sample in &samples {
                match sample.get(i) {
                    Some(value) => empty_buffer[i] += value,
                    None => (),
                }
            }
        }
        empty_buffer
    }

    let summed_array = sum_sample_array_items(vec![drums.samples, synth.samples]);

    let summed_source = AudioSample {
        samples: summed_array,
        sample_rate: drums.sample_rate,
        position: 0,
    };

    let summed_handle = Arc::new(Mutex::new(Some(summed_source)));

    let (input_stream, consumer) = get_input_stream();

    let _ = input_stream.play();

    let output = get_output_stream(summed_handle, consumer);

    let _ = output.play();

    std::thread::sleep(std::time::Duration::from_secs(10));
}
