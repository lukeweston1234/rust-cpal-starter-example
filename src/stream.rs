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

pub type RingBufConsumer<T> = Caching<Arc<SharedRb<Heap<T>>>, false, true>;

pub fn get_input_stream() -> (cpal::Stream, RingBufConsumer<f32>, RingBufConsumer<f32>) {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device");
    let input_config = input_device
        .default_input_config()
        .expect("Could not get default input config");
    println!("{} input channels!", input_config.channels());

    let sample_rate = input_config.sample_rate();

    println!("{}", sample_rate.0);

    let channels = input_config.channels();

    println!("{}", input_config.channels());

    let latency_frames = (150.0 / 1_000.0) * sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * input_config.channels() as usize;

    println!("{}", latency_samples);

    let feedback_ring = HeapRb::<f32>::new(latency_samples * 2);
    let (mut feedback_producer, feedback_consumer) = feedback_ring.split();
    for _ in 0..latency_samples {
        feedback_producer.try_push(0.0).unwrap();
    }
    let ring_recording = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer_recording, consumer_recording) = ring_recording.split();
    for _ in 0..latency_samples {
        producer_recording.try_push(0.0).unwrap();
    }
    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        data.chunks(channels as usize).for_each(|frame| {
            for &sample in frame {
                if feedback_producer.try_push(sample).is_err() {
                    output_fell_behind = true;
                }
                if producer_recording.try_push(sample).is_err() {
                    output_fell_behind = true;
                }
            }
        });
        if output_fell_behind {
            eprintln!("Output stream fell behind: try increasing latency");
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
    (input_stream, feedback_consumer, consumer_recording)
}

pub fn get_output_stream(
    mut consumer: RingBufConsumer<f32>,
) -> (cpal::Stream, Arc<MixerController>) {
    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("No output device");

    let output_stream_config = output_device
        .default_output_config()
        .expect("Could not get default output config");

    let channels = output_stream_config.channels();

    let (mixer_controller, mut mixer) = mixer();

    let process = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        data.chunks_mut(channels as usize).for_each(|frame| {
            frame.iter_mut().for_each(|sample| {
                let mixer_sample = mixer.next().unwrap_or(0.0);
                let consumer_sample = consumer.try_pop().unwrap_or(0.0);
                *sample = mixer_sample + consumer_sample;
            });
        });
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
