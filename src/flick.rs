use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

/// Conversion factor for the flick time unit
pub const FLICKS_PER_SECOND: u64 = 705_600_000;

/// Flick time unit.
///
/// See https://github.com/OculusVR/Flicks
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Flick(pub u64);

impl Flick {
    pub fn from_seconds(seconds: f64) -> Flick {
        Flick((seconds * FLICKS_PER_SECOND as f64) as u64)
    }

    pub fn from_nanoseconds(nanos: u64) -> Flick {
        Flick(nanos * 7056 / 10_000)
    }
}

impl fmt::Display for Flick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.3} s", self.0 as f32 / FLICKS_PER_SECOND as f32)
    }
}

impl From<Duration> for Flick {
    fn from(d: Duration) -> Flick {
        let nano = d.as_secs() as u64 * 1_000_000_000 + d.subsec_nanos() as u64;
        Flick::from_nanoseconds(nano)
    }
}

impl<T: Into<Flick>> Add<T> for Flick {
    type Output = Flick;

    fn add(self, rhs: T) -> Flick {
        Flick(
            self.0
                .checked_add(rhs.into().0)
                .expect("overflow when adding flicks"),
        )
    }
}

impl<T: Into<Flick>> AddAssign<T> for Flick {
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs.into();
    }
}

impl<T: Into<Flick>> Sub<T> for Flick {
    type Output = Flick;

    fn sub(self, rhs: T) -> Flick {
        Flick(
            self.0
                .checked_sub(rhs.into().0)
                .expect("overflow when subtracting flicks"),
        )
    }
}

impl<T: Into<Flick>> SubAssign<T> for Flick {
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - rhs.into();
    }
}
