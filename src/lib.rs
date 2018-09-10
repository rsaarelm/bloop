use std::cmp::max;

/// Conversion factor for the flick time unit
pub const FLICKS_PER_SECOND: u64 = 705600000;

/// Flick time unit.
///
/// See https://github.com/OculusVR/Flicks
pub type Flick = u64;

pub type Sample = i8;

pub trait Synth {
    /// Given time in flicks, return sample in [-128, 127].
    fn sample(&self, t: Flick) -> Sample;
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
            Para(a, b) => {
                max(a.duration(), b.duration())
            }
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
