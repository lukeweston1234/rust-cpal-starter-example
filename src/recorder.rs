use ringbuf::traits::Consumer;

use crate::{audio_sample::AudioSample, mixer::MixerController, stream::RingBufConsumer};
use std::sync::{mpsc::{channel, Receiver, Sender}, Arc};

pub enum RecorderMessage {
    StartRecord,
    StopRecord
}

struct RecorderController {
    sender: Sender<RecorderMessage>,
}
struct Recorder {
    consumer: RingBufConsumer<f32>,
    controller_receiver: Receiver<RecorderMessage>,
    mixer_controller: Arc<MixerController>,
    recording_buffer: Option<Vec<f32>>,
    is_recording: bool,
}
impl Recorder {
    pub fn new(consumer: RingBufConsumer<f32>, controller_receiver: Receiver<RecorderMessage>, mixer_controller: Arc<MixerController>) -> Self {
        Self {
            consumer,
            controller_receiver,
            mixer_controller,
            recording_buffer: None,
            is_recording: false
        }
    }
    pub fn start_record(&mut self){
        self.recording_buffer = Some(Vec::new());
        self.is_recording = true;
    }
    pub fn end_record(&mut self){
        self.is_recording = false;
        if let Some(completed_buffer) = self.recording_buffer.take() {
            let new_audio_sample = AudioSample::new(completed_buffer, 44_100);
            self.mixer_controller.add_audio_sample(new_audio_sample);
        } 
    }
    pub fn check_for_incoming_audio(&mut self){
        if !self.is_recording {
            return;
        }
        if let Some(sample) = self.consumer.try_pop() {
            self.recording_buffer
                .get_or_insert_with(|| vec![0.0])
                .push(sample);
        }
    }
}

pub fn recorder(consumer: RingBufConsumer<f32>, mixer_controller: Arc<MixerController>) -> (Recorder, Arc<RecorderController>) {
    let (sender, receiver) = channel::<RecorderMessage>();
    let controller = Arc::new(RecorderController { sender });
    let recorder = Recorder::new( consumer, receiver, mixer_controller);

    (recorder, controller)
}

pub fn run_recorder(mut recorder: Recorder) {
    std::thread::spawn(move || loop {
        match recorder.controller_receiver.recv() {
            Ok(msg) => match msg {
                RecorderMessage::StartRecord => recorder.start_record(),
                RecorderMessage::StopRecord => recorder.end_record(),
            },
            Err(_) => break,
        }
        recorder.check_for_incoming_audio();
    });
}