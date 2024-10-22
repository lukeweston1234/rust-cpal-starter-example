use crate::audio_sample::AudioSample;
use crate::mixer::{Mixer, MixerController};
use crate::stream::RingBufConsumer;
use ringbuf::traits::Consumer;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct RecorderController {
    is_recording: AtomicBool,
}
impl RecorderController {
    pub fn new() -> Self {
        Self {
            is_recording: AtomicBool::new(false),
        }
    }
    pub fn start_recording(&self) {
        self.is_recording
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn stop_recording(&self) {
        self.is_recording
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct Recorder {
    consumer: RingBufConsumer<f32>,
    recorder_controller: Arc<RecorderController>,
    mixer_controller: Arc<MixerController>,
    current_recording_clip: Option<Vec<f32>>,
}
impl Recorder {
    pub fn new(
        consumer: RingBufConsumer<f32>,
        recorder_controller: Arc<RecorderController>,
        mixer_controller: Arc<MixerController>,
    ) -> Self {
        Self {
            consumer,
            recorder_controller,
            mixer_controller,
            current_recording_clip: None,
        }
    }
}

pub fn run_recorder(mut recorder: Recorder) {
    std::thread::spawn(move || loop {
        match recorder.consumer.try_pop() {
            Some(sample) => {
                if recorder
                    .recorder_controller
                    .is_recording
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    if let Some(clip) = recorder.current_recording_clip.as_mut() {
                        clip.push(sample);
                    } else {
                        recorder.current_recording_clip = Some(vec![sample]);
                    }
                } else {
                    if let Some(clip) = recorder.current_recording_clip.take() {
                        recorder
                            .mixer_controller
                            .add_audio_sample(AudioSample::new(clip, 44_100));
                    };
                }
            }
            None => (),
        }
    });
}

pub fn recorder(
    consumer: RingBufConsumer<f32>,
    mixer_controller: Arc<MixerController>,
) -> (Recorder, Arc<RecorderController>) {
    let recorder_controller = Arc::new(RecorderController::new());
    let recorder = Recorder::new(consumer, recorder_controller.clone(), mixer_controller);
    (recorder, recorder_controller)
}
