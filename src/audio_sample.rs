pub struct AudioSample {
    samples: Vec<f32>,
    sample_rate: u32,
}
impl AudioSample {
    pub fn new(samples: Vec<f32>, sample_rate: u32, position: usize) -> Self {
        Self {
            samples,
            sample_rate,
        }
    }
    pub fn get(&self, position: usize) -> Option<&f32> {
        return self.samples.get(position);
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
    })
}
