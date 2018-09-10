extern crate cpal;
extern crate bloop;

use bloop::{Synth, Flick, Sample, Music, Primitive};
use std::f64::consts::PI;

struct SineWave {
    pitch: f64,
    volume: f64,
}

impl Synth for SineWave {
    fn sample(&self, t: Flick) -> Sample {
        (self.volume * (t as f64 * self.pitch / bloop::FLICKS_PER_SECOND as f64 * 2.0 * PI).sin() * 127.0)
            as Sample
    }
}

fn main() {
    let device = cpal::default_output_device().expect("Failed to get default output device");
    let format = device.default_output_format().expect("Failed to get default output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let mut tick = 0u64;
    let rate_multiplier = bloop::FLICKS_PER_SECOND / format.sample_rate.0 as u64;

    let music = Music::Prim(Primitive::Note(
            5 * bloop::FLICKS_PER_SECOND, SineWave { pitch: 440.0, volume: 0.5 }));

    // Produce a sinusoid of maximum amplitude.
    let mut next_value = || {
        tick += 1;
        let sample = music.sample(tick * rate_multiplier);
        (sample as f32) / 128.0
    };

    event_loop.run(move |_, data| {
        match data {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = (next_value() * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = next_value();
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            _ => (),
        }
    });
}
