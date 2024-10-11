use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    OutputCallbackInfo, Sample,
};

fn main() {
    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("No output device");

    let stream_config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(44100),
        buffer_size: cpal::BufferSize::Default,
    };

    let process = move |data: &mut [f32], _: &OutputCallbackInfo| {
        for sample in data.iter_mut() {
            *sample = Sample::EQUILIBRIUM;
        }
    };

    let stream = output_device
        .build_output_stream(
            &stream_config,
            process,
            |err| eprintln!("An error occured in the output stream {}", err),
            None,
        )
        .expect("Could not build output stream");

    stream.play().expect("Could not play stream!");

    std::thread::sleep(std::time::Duration::from_secs(3));
}
