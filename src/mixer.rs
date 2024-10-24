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
        let audio_store = self.audio_store.lock().unwrap();

        // Determine the number of channels; assuming all samples have the same channel count
        let channels = if !audio_store.is_empty() {
            audio_store[0].channels as usize
        } else {
            1
        };

        // Find the maximum number of frames (total samples divided by channels)
        let max_frames = audio_store
            .iter()
            .map(|x| x.samples.len() / channels)
            .max()
            .unwrap_or(0);

        let mut prepared_audio = self.prepared_audio.lock().unwrap();
        prepared_audio.resize(max_frames * channels, 0.0);

        // Sum the samples per frame per channel
        for frame_index in 0..max_frames {
            for channel in 0..channels {
                let sample_index = frame_index * channels + channel;
                let mut sample_sum = 0.0;
                for audio_sample in audio_store.iter() {
                    if audio_sample.channels as usize != channels {
                        // Handle mismatched channels if needed
                        // For now, skip or handle accordingly
                        continue;
                    }
                    let sample = audio_sample
                        .samples
                        .get(sample_index)
                        .copied()
                        .unwrap_or(0.0);
                    sample_sum += sample;
                }
                prepared_audio[sample_index] = sample_sum;
            }
        }
        if prepared_audio.len() % channels != 0 {
            prepared_audio.push(0.0);
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

        if self.position >= self.audio_buffer.len() {
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
