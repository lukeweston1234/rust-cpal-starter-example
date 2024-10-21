use crate::stream::RingBufConsumer;

struct Recorder {
    consumer: RingBufConsumer<f32>,
}
