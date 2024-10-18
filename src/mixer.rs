use std::sync::{atomic::AtomicBool, Arc, Mutex};

use crate::audio_sample::AudioSample;

pub struct MixerController {
    audio_store: Vec<AudioSample>,
    has_prepared_audio: AtomicBool,
    prepared_audio: Vec<f32>,
}
impl MixerController {
    pub fn new() -> Self {
        Self {
            audio_store: Vec::new(),
            has_prepared_audio: AtomicBool::new(false),
            prepared_audio: Vec::new(),
        }
    }
    pub fn add_audio_sample(&mut self, audio_sample: AudioSample) {
        self.audio_store.push(audio_sample);
        self.sum_audio_store();
    }
    pub fn remove_audio_sample(&mut self, index: usize) {
        self.audio_store.remove(index);
        self.sum_audio_store();
    }
    pub fn sum_audio_store(&mut self) {
        let max_length = self
            .audio_store
            .iter()
            .map(|x| x.get_samples().len())
            .max()
            .unwrap_or(0);
        let mut new_audio_buffer = vec![0.0; max_length];
        for i in 0..max_length - 1 {
            for sample in &self.audio_store {
                match sample.get_samples().get(i) {
                    Some(value) => new_audio_buffer[i] += value,
                    None => (),
                }
            }
        }
        self.prepared_audio = new_audio_buffer;
        self.has_prepared_audio
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn get_prepared_audio(&self) -> &Vec<f32> {
        &self.prepared_audio
    }
}

pub struct Mixer {
    audio_buffer: Vec<f32>,
    controller: Arc<MixerController>,
    position: usize,
    is_looping: bool,
}
impl Mixer {
    pub fn new(controller: Arc<MixerController>, is_looping: bool) -> Self {
        Self {
            audio_buffer: Vec::new(),
            controller: controller,
            position: 0,
            is_looping,
        }
    }
}
impl Iterator for Mixer {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self
            .controller
            .has_prepared_audio
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.audio_buffer = self.controller.get_prepared_audio().to_vec(); // TODO: How bad is this approach?
        }
        if self.position > self.audio_buffer.len() {
            if self.is_looping {
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

pub fn mixer() -> (Mixer, Arc<MixerController>) {
    let controller = Arc::new(MixerController::new());
    let mixer = Mixer::new(controller.clone(), false);

    return (mixer, controller);
}
