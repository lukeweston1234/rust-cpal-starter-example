use crate::audio_sample::AudioSample;
use crate::mixer::MixerController;
use crate::player::PlayerController;
use crate::stream::RingBufConsumer;
use ringbuf::traits::Consumer;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// pub enum RecorderMessage {
//     ClipAdded,
// }

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
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn stop_recording(&self) {
        self.is_recording
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

pub struct Recorder {
    consumer: RingBufConsumer<f32>,
    recorder_controller: Arc<RecorderController>,
    mixer_controller: Arc<MixerController>,
    current_recording_clip: Option<Vec<f32>>,
    position: usize,
    buffer_size: usize,
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
            position: 0,
            buffer_size: (44_100 * 4 * 2), // 60 bpm 4 bars at 44100hz 2 channels
        }
    }
}

pub fn run_recorder(mut recorder: Recorder, player_controller: Arc<PlayerController>) {
    std::thread::spawn(move || loop {
        match recorder.consumer.try_pop() {
            Some(sample) => {
                if recorder
                    .recorder_controller
                    .is_recording
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
                    if recorder.position >= recorder.buffer_size {
                        if let Some(audio_sample) = recorder.current_recording_clip.take() {
                            recorder.mixer_controller.add_audio_sample(AudioSample::new(
                                audio_sample,
                                44_100,
                                2,
                            ));
                            recorder.position = 0;
                            player_controller.on_clip_add();
                        }
                    }
                    if let Some(clip) = recorder.current_recording_clip.as_mut() {
                        clip.push(sample);
                    } else {
                        recorder.current_recording_clip = Some(vec![sample]);
                    }
                    recorder.position += 1;
                } else {
                    if let Some(clip) = recorder.current_recording_clip.take() {
                        recorder
                            .mixer_controller
                            .add_audio_sample(AudioSample::new(clip, 44_100, 2));
                        recorder.position = 0;
                        player_controller.on_clip_add();
                    }
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
