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

use crate::mixer::{mixer, MixerController};

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

pub fn get_output_stream(mut consumer: RingBufConsumer) -> (cpal::Stream, Arc<MixerController>) {
    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("No output device");

    let output_stream_config = output_device
        .default_output_config()
        .expect("Could not get default output config");

    let channels = output_stream_config.channels();

    let (mixer_controller, mut mixer) = mixer();

    let process = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        data.chunks_mut(channels.into()).for_each(|frame| {
            for sample in frame.iter_mut() {
                let consumer_sample = match consumer.try_pop() {
                    Some(s) => s,
                    None => 0.0,
                };
                *sample = mixer.next().unwrap_or(0f32) + consumer_sample;
            }
        })
    };

    let output_stream = output_device
        .build_output_stream(
            &output_stream_config.into(),
            process,
            |err| eprintln!("{err}"),
            None,
        )
        .expect("Could not build output stream!");

    (output_stream, mixer_controller)
}
