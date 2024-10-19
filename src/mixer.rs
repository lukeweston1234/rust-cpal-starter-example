use std::sync::{atomic::AtomicBool, Arc, Mutex};

use crate::audio_sample::AudioSample;

pub struct MixerController {
    audio_store: Mutex<Vec<AudioSample>>,
    has_prepared_audio: AtomicBool,
    prepared_audio: Mutex<Vec<f32>>,
    is_looping: AtomicBool,
}
impl MixerController {
    pub fn new() -> Self {
        Self {
            audio_store: Mutex::new(Vec::new()),
            has_prepared_audio: AtomicBool::new(false),
            prepared_audio: Mutex::new(Vec::new()),
            is_looping: AtomicBool::new(true),
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
        for i in 0..max_length - 1 {
            for sample in self.audio_store.lock().unwrap().iter() {
                match sample.get_samples().get(i) {
                    Some(value) => prepared_audio[i] += value,
                    None => (),
                }
            }
        }
        self.has_prepared_audio
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn get_prepared_audio(&self) -> Vec<f32> {
        self.prepared_audio.lock().unwrap().to_vec()
    }
    pub fn set_prepared_false(&self) {
        self.has_prepared_audio
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn set_is_looping(&self, new_val: bool) {
        self.is_looping
            .store(new_val, std::sync::atomic::Ordering::Relaxed);
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
        if self
            .controller
            .has_prepared_audio
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.audio_buffer = self.controller.get_prepared_audio().to_vec(); // TODO: How bad is this approach?
            self.controller.set_prepared_false();
        }
        if self.position > self.audio_buffer.len() {
            if self
                .controller
                .is_looping
                .load(std::sync::atomic::Ordering::Relaxed)
            {
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
    let controller = Arc::new(MixerController::new());
    let mixer = Mixer::new(controller.clone());

    (controller, mixer)
}
