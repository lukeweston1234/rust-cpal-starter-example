use audio_sample::{load_wav, sum_audio_clips, AudioSample};
use cpal::{self, traits::StreamTrait};
use std::sync::{Arc, Mutex};
use stream::{get_input_stream, get_output_stream};

mod audio_sample;
mod stream;

type SummedAudioHandle = Arc<Mutex<Option<AudioSample>>>;

fn main() {
    let drums = load_wav("assets/drums_32.wav").expect("Could not load drums!");
    let synth = load_wav("assets/synth_32.wav").expect("Could not load synth!");

    println!("Samples loaded!");

    let summed_source = sum_audio_clips(vec![drums, synth]);

    let summed_handle = Arc::new(Mutex::new(Some(summed_source)));

    let (input_stream, consumer) = get_input_stream();

    let _ = input_stream.play();

    let output = get_output_stream(summed_handle, consumer);

    let _ = output.play();

    std::thread::sleep(std::time::Duration::from_secs(10));
}
