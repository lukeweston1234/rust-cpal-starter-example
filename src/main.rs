use audio_sample::load_wav;
use cpal::{self, traits::StreamTrait};
use player::{player, run_player};
use stream::{get_input_stream, get_output_stream};

mod audio_sample;
mod mixer;
mod player;
mod stream;

fn main() {
    let drums = load_wav("assets/drums_32.wav").expect("Could not load drums!");
    // let synth = load_wav("assets/synth_32.wav").expect("Could not load synth!");

    println!("Samples loaded!");

    let (input_stream, consumer) = get_input_stream();

    let (output_stream, mixer_controller) = get_output_stream(consumer);

    mixer_controller.add_audio_sample(drums);

    let _ = output_stream.play();

    let _ = input_stream.play();

    let (player, player_controller) = player(mixer_controller);

    run_player(player);

    player_controller.play();

    std::thread::sleep(std::time::Duration::from_secs(3));

    player_controller.pause();

    std::thread::sleep(std::time::Duration::from_secs(3));

    player_controller.play();

    std::thread::sleep(std::time::Duration::from_secs(3));

    player_controller.stop();

    std::thread::sleep(std::time::Duration::from_secs(1));

    player_controller.play();

    loop {}
}
