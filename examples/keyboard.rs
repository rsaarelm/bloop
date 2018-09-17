extern crate bloop;
extern crate sdl2;

use bloop::{Flick, Sample, Synth, FLICKS_PER_SECOND};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashSet;
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

struct Piano {
    start: Flick,
    end: Option<Flick>,
}

impl Piano {
    pub fn new() -> Piano {
        Piano {
            start: Flick(std::u64::MAX / 2),
            end: None,
        }
    }
}

impl Synth for Piano {
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
        let pitch = 440.0;

        (volume * (t.0 as f64 * pitch / bloop::FLICKS_PER_SECOND as f64 * 2.0 * PI).sin() * 127.0)
            as Sample
    }
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

    let mut piano = Arc::new(Mutex::new(Piano::new()));
    let mut t_zero = SystemTime::now();

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
                println!("down {}", t);
                let mut piano = piano.lock().unwrap();
                piano.start = t;
                piano.end = None;
            }

            if let Event::KeyUp {
                scancode: Some(scan),
                repeat: false,
                ..
            } = event
            {
                println!("  up {}", t);
                let mut piano = piano.lock().unwrap();
                piano.end = Some(t);
            }
        }
    }
}
