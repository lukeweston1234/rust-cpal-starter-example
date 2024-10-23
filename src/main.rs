use audio_sample::{load_wav, AudioSample};
use cpal::{self, traits::StreamTrait};
use player::{player, run_player};
use recorder::{recorder, run_recorder};
use stream::{get_input_stream, get_output_stream};

mod audio_sample;
mod mixer;
mod player;
mod recorder;
mod stream;

fn main() {
    let drums = load_wav("assets/drums_32.wav").expect("Could not load drums!");
    let synth = load_wav("assets/synth_32.wav").expect("Could not load synth!");

    println!("Samples loaded!");

    let (input_stream, consumer, recording_consumer) = get_input_stream();

    let (output_stream, mixer_controller) = get_output_stream(consumer);

    // let zero_vector = AudioSample::zero_buffer(44_100, 120, 4, 4, 2);

    // mixer_controller.add_audio_sample(zero_vector);

    // mixer_controller.add_audio_sample(drums);

    let recorder_mixer = mixer_controller.clone();

    let (recorder, recorder_controller) = recorder(recording_consumer, recorder_mixer);

    output_stream.play().expect("Could not play output");

    input_stream.play().expect("Could not play input");

    let (player, player_controller) = player(mixer_controller, recorder_controller);

    run_player(player);

    run_recorder(recorder, player_controller.clone());

    player_controller.play();

    player_controller.record();

    loop {}
}
