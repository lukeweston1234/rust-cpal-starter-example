use cpal::{
    self,
    traits::{DeviceTrait, HostTrait},
};
use ringbuf::{
    storage::Heap,
    traits::{Consumer, Producer, Split},
    wrap::caching::Caching,
    HeapRb, SharedRb,
};
use std::sync::Arc;

use crate::SummedAudioHandle;

pub type RingBufConsumer = Caching<Arc<SharedRb<Heap<f32>>>, false, true>;

pub fn get_input_stream() -> (cpal::Stream, RingBufConsumer) {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device");
    let input_config = input_device
        .default_input_config()
        .expect("Could not get default input config");
    println!("{} input channels!", input_config.channels());
    let ring = HeapRb::<f32>::new(1024);
    let (mut producer, consumer) = ring.split();
    for _ in 0..1024 {
        producer.try_push(0.0).unwrap();
    }
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if producer.try_push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };
    let input_stream = input_device
        .build_input_stream(
            &input_config.into(),
            input_data_fn,
            |err| eprint!("Error occured in input stream {}", err),
            None,
        )
        .expect("Could not create input stream");
    (input_stream, consumer)
}

pub fn get_output_stream(
    summed_handle: SummedAudioHandle,
    mut consumer: RingBufConsumer,
) -> cpal::Stream {
    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("No output device");

    let output_stream_config = output_device
        .default_output_config()
        .expect("Could not get default output config");

    let channels = output_stream_config.channels();

    let process = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        if let Ok(mut guard) = summed_handle.try_lock() {
            if let Some(audio_sample) = guard.as_mut() {
                for frame in data.chunks_mut(channels.into()) {
                    for sample in frame.iter_mut() {
                        // Directly use consumer here
                        let input_buf_sample = match consumer.try_pop() {
                            Some(s) => s,
                            None => 0.0,
                        };

                        let memory_sample = audio_sample
                            .get(audio_sample.get_position())
                            .cloned()
                            .unwrap_or(0.0);

                        *sample = cpal::Sample::from_sample(memory_sample + input_buf_sample);

                        audio_sample.increment_position();
                    }
                }
            }
        }
    };

    let output_stream = output_device
        .build_output_stream(
            &output_stream_config.into(),
            process,
            |err| eprintln!("{err}"),
            None,
        )
        .expect("Could not build output stream!");

    output_stream
}
