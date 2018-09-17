use cpal;
use std::sync::{Arc, Mutex};
use std::thread;

use {Flick, Synth, FLICKS_PER_SECOND};

/// Spin off a player thread.
pub fn spawn_cpal_player<S: Synth + 'static>(synth: Arc<Mutex<S>>) {
    let device = cpal::default_output_device().expect("Failed to get default output device");
    let format = device
        .default_output_format()
        .expect("Failed to get default output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let mut tick = 0u64;
    let rate_multiplier = FLICKS_PER_SECOND / format.sample_rate.0 as u64;

    thread::spawn(move || {
        event_loop.run(move |_, data| {
            let synth = synth.lock().unwrap();

            let mut next_value = || {
                tick += 1;
                let sample = synth.sample(Flick(tick * rate_multiplier));
                (sample as f32) / 128.0
            };

            match data {
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = ((next_value() * 0.5 + 0.5) * ::std::u16::MAX as f32) as u16;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = (next_value() * ::std::i16::MAX as f32) as i16;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = next_value();
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
                _ => (),
            }
        });
    });
}
