use std::sync::{
    atomic::{AtomicBool, AtomicU8},
    Arc, Mutex,
};

use crate::audio_sample::AudioSample;

#[derive(PartialEq, Debug)]
pub enum MixerState {
    PlayingOneShot,
    PlayingLooping,
    Paused,
    Stopped,
}

pub struct MixerController {
    audio_store: Mutex<Vec<AudioSample>>,
    has_prepared_audio: AtomicBool,
    prepared_audio: Mutex<Vec<f32>>,
    mixer_state: AtomicU8,
}
impl MixerController {
    pub fn new() -> Self {
        Self {
            audio_store: Mutex::new(Vec::new()),
            has_prepared_audio: AtomicBool::new(false),
            prepared_audio: Mutex::new(Vec::new()),
            mixer_state: AtomicU8::new(MixerState::Stopped as u8),
        }
    }
    pub fn add_audio_sample(&self, audio_sample: AudioSample) {
        self.audio_store.lock().unwrap().push(audio_sample);
        self.sum_audio_store();
    }
    pub fn remove_audio_sample(&mut self, index: usize) {
        self.audio_store.lock().unwrap().remove(index);
        self.sum_audio_store();
    }
    pub fn sum_audio_store(&self) {
        println!("Summing audio!");
        let max_length = self
            .audio_store
            .lock()
            .unwrap()
            .iter()
            .map(|x| x.get_samples().len())
            .max()
            .unwrap_or(0);
        let mut prepared_audio = self.prepared_audio.lock().unwrap();
        prepared_audio.resize(max_length, 0.0);
        let audio_store = self.audio_store.lock().unwrap();
        for i in 0..max_length - 1 {
            for sample in audio_store.iter() {
                match sample.get_samples().get(i) {
                    Some(value) => prepared_audio[i] += value,
                    None => (),
                }
            }
        }
        self.has_prepared_audio
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn get_prepared_audio(&self) -> Vec<f32> {
        self.prepared_audio.lock().unwrap().to_vec()
    }
    pub fn set_prepared_false(&self) {
        self.has_prepared_audio
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn set_mixer_state(&self, mixer_state: MixerState) {
        println!("{:?}", mixer_state);
        self.mixer_state
            .store(mixer_state as u8, std::sync::atomic::Ordering::SeqCst);
    }
}

pub struct Mixer {
    audio_buffer: Vec<f32>,
    controller: Arc<MixerController>,
    position: usize,
}
impl Mixer {
    pub fn new(controller: Arc<MixerController>) -> Self {
        Self {
            audio_buffer: Vec::new(),
            controller: controller,
            position: 0,
        }
    }
}
impl Iterator for Mixer {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let mixer_state_num = self
            .controller
            .mixer_state
            .load(std::sync::atomic::Ordering::SeqCst);

        let mixer_state = match mixer_state_num {
            0 => MixerState::PlayingOneShot,
            1 => MixerState::PlayingLooping,
            2 => MixerState::Paused,
            3 => MixerState::Stopped,
            _ => MixerState::Stopped,
        };

        if mixer_state == MixerState::Stopped {
            self.position = 0; // The fact that were doing this over and over again, means we need to change something
            return None;
        }

        if mixer_state == MixerState::Paused {
            return None;
        }

        if self
            .controller
            .has_prepared_audio
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            self.audio_buffer = self.controller.get_prepared_audio().to_vec(); // TODO: How bad is this approach?
            self.controller.set_prepared_false();
        }

        if self.position > self.audio_buffer.len() {
            if mixer_state == MixerState::PlayingLooping {
                self.position = 0;
            } else {
                return None;
            }
        }

        let sample = self.audio_buffer.get(self.position).copied();
        self.position += 1;
        sample
    }
}

pub fn mixer() -> (Arc<MixerController>, Mixer) {
    let mixer_controller = Arc::new(MixerController::new());
    let mixer = Mixer::new(mixer_controller.clone());

    (mixer_controller, mixer)
}
