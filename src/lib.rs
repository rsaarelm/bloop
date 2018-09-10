/// Time in nanoseconds.
pub type Duration = f64;

pub trait Synth {
    /// Given time in seconds, return sample in [-1.0, 1.0].
    fn sample(&self, t: Duration) -> f32;
}

pub enum Primitive<T: Synth> {
    Note(Duration, T),
    Rest(Duration),
}

impl<T: Synth> Primitive<T> {
    fn duration(&self) -> Duration {
        use Primitive::*;
        match self {
            Note(d, _) => *d,
            Rest(d) => *d,
        }
    }
}

impl<T: Synth> Synth for Primitive<T> {
    fn sample(&self, t: Duration) -> f32 {
        use Primitive::*;
        match self {
            Note(d, a) => if t <= *d { a.sample(t) } else { 0.0 }
            Rest(_) => 0.0
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
    fn duration(&self) -> Duration {
        use Music::*;
        match self {
            Prim(p) => p.duration(),
            Para(a, b) => {
                // XXX: max-function equivalent for floats?
                let (a, b) = (a.duration(), b.duration());
                if a > b { a } else { b }
            }
            Seq(a, b) => a.duration() + b.duration(),
            Modify(c, a) => unimplemented!()
        }
    }
}

impl<T: Synth> Synth for Music<T> {
    fn sample(&self, t: Duration) -> f32 {
        use Music::*;
        match self {
            Prim(p) => clamp(p.sample(t)),
            Para(a, b) => clamp(a.sample(t) + b.sample(t)),
            Seq(a, b) => {
                let t1 = a.duration();
                if t <= t1 {
                    a.sample(t)
                } else {
                    b.sample(t - t1)
                }
            }
            Modify(c, a) => unimplemented!()
        }
    }
}

fn clamp(a: f32) -> f32 {
    if a < -1.0 {
        -1.0
    } else if a > 1.0 {
        1.0
    } else {
        a
    }
}
