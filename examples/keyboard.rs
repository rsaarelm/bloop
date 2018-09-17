extern crate bloop;
extern crate sdl2;

use bloop::{Flick, Note, Sample, Synth};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::ops::{Add, Mul, Sub};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Interpolate linearly between two values.
fn lerp<T, U, V, W>(a: U, b: U, t: T) -> W
where
    U: Add<V, Output = W> + Sub<U, Output = V> + Copy,
    V: Mul<T, Output = V>,
{
    a + (b - a) * t
}

struct ADSR {
    pub a: Flick,
    pub d: Flick,
    pub s: f64,
    pub r: Flick,
}

impl ADSR {
    pub fn new(a: Flick, d: Flick, s: f64, r: Flick) -> ADSR {
        ADSR { a, d, s, r }
    }

    /// Return ADSR envelope coefficient given time point.
    pub fn get(&self, t: Flick, end: Option<Flick>) -> f64 {
        if t < self.a {
            return (t.0 as f64) / (self.a.0 as f64);
        }

        if (t - self.a) < self.d {
            return lerp(1.0, self.s, (t - self.a).0 as f64 / self.d.0 as f64);
        }

        if let Some(end) = end {
            if t > end {
                let mut level = lerp(self.s, 0.0, (t - end).0 as f64 / self.r.0 as f64);
                if level < 0.0 {
                    level = 0.0;
                }
                return level;
            }
        }

        self.s
    }
}

struct Channel {
    pitch: f64,
    start: Flick,
    end: Option<Flick>,
}

impl Channel {
    pub fn new(pitch: f64) -> Channel {
        Channel {
            pitch,
            start: Flick(std::u64::MAX / 2),
            end: None,
        }
    }
}

impl Synth for Channel {
    fn sample(&self, t: Flick) -> Sample {
        let adsr = ADSR::new(
            Flick::from_seconds(1.0 / 8.0),
            Flick::from_seconds(1.0 / 6.0),
            0.6,
            Flick::from_seconds(1.0 / 4.0),
        );

        let volume = if t < self.start {
            0.0
        } else {
            adsr.get(
                t - self.start,
                self.end.map(|x| Flick(x.0.saturating_sub(self.start.0))),
            )
        };

        (volume
            * (t.0 as f64 * self.pitch / bloop::FLICKS_PER_SECOND as f64 * 2.0 * PI).sin()
            * 127.0) as Sample
    }
}

fn piano() -> (Vec<Channel>, HashMap<Scancode, usize>) {
    use Scancode::*;

    let mut ret = (Vec::new(), HashMap::new());

    for (i, (k, note, octave)) in [
        // TODO: Use notes
        (Z, Note::C, 4),
        (S, Note::Cs, 4),
        (X, Note::D, 4),
        (D, Note::Ds, 4),
        (C, Note::E, 4),
        (V, Note::F, 4),
        (G, Note::Fs, 4),
        (B, Note::G, 4),
        (H, Note::Gs, 4),
        (N, Note::A, 5),
        (J, Note::As, 5),
        (M, Note::B, 5),
        (Comma, Note::C, 5),
        (L, Note::Cs, 5),
        (Period, Note::D, 5),
        (Semicolon, Note::Ds, 5),
        (Slash, Note::E, 5),
        (Q, Note::C, 5),
        (Num2, Note::Cs, 5),
        (W, Note::D, 5),
        (Num3, Note::Ds, 5),
        (E, Note::E, 5),
        (R, Note::F, 5),
        (Num5, Note::Fs, 5),
        (T, Note::G, 5),
        (Num6, Note::Gs, 5),
        (Y, Note::A, 6),
        (Num7, Note::As, 6),
        (U, Note::B, 6),
        (I, Note::C, 6),
        (Num9, Note::Cs, 6),
        (O, Note::D, 6),
        (Num0, Note::Ds, 6),
        (P, Note::E, 6),
    ]
        .iter()
        .enumerate()
    {
        ret.0.push(Channel::new(note.freq(*octave)));
        ret.1.insert(*k, i);
    }

    ret
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let _window = video_subsystem
        .window("Keyboard", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut events = sdl_context.event_pump().unwrap();

    let (piano, keymap) = piano();
    let piano = Arc::new(Mutex::new(piano));

    let t_zero = SystemTime::now();

    bloop::spawn_cpal_player(piano.clone());

    'running: loop {
        for event in events.poll_iter() {
            let t = Flick::from(t_zero.elapsed().unwrap());

            if let Event::Quit { .. } = event {
                break 'running;
            }

            if let Event::KeyDown {
                scancode: Some(scan),
                repeat: false,
                ..
            } = event
            {
                if let Some(&idx) = keymap.get(&scan) {
                    let mut piano = piano.lock().unwrap();
                    piano[idx].start = t;
                    piano[idx].end = None;
                }
            }

            if let Event::KeyUp {
                scancode: Some(scan),
                repeat: false,
                ..
            } = event
            {
                if let Some(&idx) = keymap.get(&scan) {
                    let mut piano = piano.lock().unwrap();
                    piano[idx].end = Some(t);
                }
            }
        }
    }
}
