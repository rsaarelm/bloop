extern crate cpal;

use std::cmp::max;

mod flick;
pub use flick::{Flick, FLICKS_PER_SECOND};

mod play;
pub use play::spawn_cpal_player;

pub type Sample = i8;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Note {
    A,
    As,
    B,
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
}

impl Note {
    pub fn freq(self, octave: i32) -> f64 {
        27.5 * 2.0f64.powi(octave) * 2.0f64.powf(self as usize as f64 / 12.0)
    }
}

pub trait Synth: Send {
    /// Given time in flicks, return sample in [-128, 127].
    fn sample(&self, t: Flick) -> Sample;
}

impl<T: Synth> Synth for Vec<T> {
    fn sample(&self, t: Flick) -> Sample {
        self.iter()
            .fold(0, |acc, s| acc.saturating_add(s.sample(t)))
    }
}

pub enum Primitive<T: Synth> {
    Note(Flick, T),
    Rest(Flick),
}

impl<T: Synth> Primitive<T> {
    fn duration(&self) -> Flick {
        use Primitive::*;
        match self {
            Note(d, _) => *d,
            Rest(d) => *d,
        }
    }
}

impl<T: Synth> Synth for Primitive<T> {
    fn sample(&self, t: Flick) -> Sample {
        use Primitive::*;
        match self {
            Note(d, a) => if t <= *d {
                a.sample(t)
            } else {
                0
            },
            Rest(_) => 0,
        }
    }
}

pub enum Control {
    Tempo(f64),
    Transpose(f64),
}

pub enum Music<T: Synth> {
    Prim(Primitive<T>),
    Para(Box<Music<T>>, Box<Music<T>>),
    Seq(Box<Music<T>>, Box<Music<T>>),
    Modify(Control, Box<Music<T>>),
}

impl<T: Synth> Music<T> {
    fn duration(&self) -> Flick {
        use Music::*;
        match self {
            Prim(p) => p.duration(),
            Para(a, b) => max(a.duration(), b.duration()),
            Seq(a, b) => a.duration() + b.duration(),
            Modify(c, a) => unimplemented!(),
        }
    }
}

impl<T: Synth> Synth for Music<T> {
    fn sample(&self, t: Flick) -> Sample {
        use Music::*;
        match self {
            Prim(p) => p.sample(t),
            Para(a, b) => a.sample(t).saturating_add(b.sample(t)),
            Seq(a, b) => {
                let t1 = a.duration();
                if t <= t1 {
                    a.sample(t)
                } else {
                    b.sample(t - t1)
                }
            }
            Modify(c, a) => unimplemented!(),
        }
    }
}
