pub struct AudioSample {
    samples: Vec<f32>,
    sample_rate: u32,
    position: usize,
}
impl AudioSample {
    pub fn new(samples: Vec<f32>, sample_rate: u32, position: usize) -> Self {
        Self {
            samples,
            sample_rate,
            position,
        }
    }
    pub fn get(&self, position: usize) -> Option<&f32> {
        return self.samples.get(position);
    }
    pub fn get_position(&self) -> usize {
        return self.position;
    }
    pub fn increment_position(&mut self) {
        self.position += 1;
    }
    pub fn get_samples(&self) -> &Vec<f32> {
        return &self.samples;
    }
}

pub fn load_wav(file_path: &str) -> Result<AudioSample, hound::Error> {
    let reader = hound::WavReader::open(file_path)?;
    let spec = reader.spec();
    println!("WAV Spec: {:?}", spec);

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .collect::<Result<Vec<f32>, _>>()?,
        hound::SampleFormat::Int => reader
            .into_samples::<i32>()
            .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
            .collect::<Result<Vec<f32>, _>>()?,
    };

    Ok(AudioSample {
        samples,
        sample_rate: spec.sample_rate,
        position: 0,
    })
}

pub fn sum_audio_clips(samples: Vec<AudioSample>) -> AudioSample {
    let max_length = samples.iter().map(|x| x.samples.len()).max().unwrap_or(0);
    let mut new_audio_buffer = vec![0.0; max_length];
    for i in 0..max_length - 1 {
        for sample in &samples {
            match sample.samples.get(i) {
                Some(value) => new_audio_buffer[i] += value,
                None => (),
            }
        }
    }
    AudioSample {
        samples: new_audio_buffer,
        sample_rate: samples[0].sample_rate, // TODO: Generic
        position: 0,
    }
}
