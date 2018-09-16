extern crate bloop;

use std::sync::{Arc, Mutex};
use bloop::{Synth, Flick, Sample, Music, Primitive};
use std::f64::consts::PI;
use std::{thread, time};

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
    let music = Music::Prim(Primitive::Note(
            5 * bloop::FLICKS_PER_SECOND, SineWave { pitch: 440.0, volume: 0.5 }));

    bloop::spawn_cpal_player(Arc::new(Mutex::new(music)));
    thread::sleep(time::Duration::from_secs(5));
}
