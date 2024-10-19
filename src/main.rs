use audio_sample::{load_wav, sum_audio_clips, AudioSample};
use cpal::{self, traits::StreamTrait};
use std::sync::{Arc, Mutex};
use stream::{get_input_stream, get_output_stream};

mod audio_sample;
mod mixer;
mod player;
mod stream;

fn main() {
    let drums = load_wav("assets/drums_32.wav").expect("Could not load drums!");
    let synth = load_wav("assets/synth_32.wav").expect("Could not load synth!");

    println!("Samples loaded!");

    let (input_stream, consumer) = get_input_stream();

    let _ = input_stream.play();

    let (output_stream, controller) = get_output_stream(consumer);

    controller.set_is_looping(false);

    controller.add_audio_sample(drums);

    let _ = output_stream.play();

    loop {}
}
