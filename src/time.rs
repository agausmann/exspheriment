//! Custom monotonic time system inspired by std::time, but not fixed to
//! real-time.

use std::ops::{Add, AddAssign, Rem, RemAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimInstant {
    micros: i64,
}

impl SimInstant {
    pub fn epoch() -> Self {
        Self { micros: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimDuration {
    micros: i64,
}

impl SimDuration {
    pub fn from_micros(micros: i64) -> Self {
        Self { micros }
    }

    pub fn as_micros(&self) -> i64 {
        self.micros
    }

    pub fn from_secs_f64(secs: f64) -> Self {
        Self::from_micros((secs * 1.0e6) as i64)
    }

    pub fn as_secs_f64(&self) -> f64 {
        (self.micros as f64) * 1.0e-6
    }

    pub fn as_secs_f32(&self) -> f32 {
        (self.micros as f32) * 1.0e-6
    }
}

impl From<std::time::Duration> for SimDuration {
    fn from(duration: std::time::Duration) -> SimDuration {
        SimDuration::from_micros(duration.as_micros().try_into().expect("duration overflow"))
    }
}

impl Add<SimDuration> for SimInstant {
    type Output = SimInstant;

    fn add(self, rhs: SimDuration) -> Self {
        Self {
            micros: self.micros + rhs.micros,
        }
    }
}

impl AddAssign<SimDuration> for SimInstant {
    fn add_assign(&mut self, rhs: SimDuration) {
        self.micros += rhs.micros;
    }
}

impl Sub for SimInstant {
    type Output = SimDuration;

    fn sub(self, rhs: Self) -> SimDuration {
        SimDuration {
            micros: self.micros - rhs.micros,
        }
    }
}

impl Sub<SimDuration> for SimInstant {
    type Output = SimInstant;

    fn sub(self, rhs: SimDuration) -> Self {
        Self {
            micros: self.micros - rhs.micros,
        }
    }
}

impl SubAssign<SimDuration> for SimInstant {
    fn sub_assign(&mut self, rhs: SimDuration) {
        self.micros -= rhs.micros;
    }
}

impl Add for SimDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            micros: self.micros + rhs.micros,
        }
    }
}

impl AddAssign for SimDuration {
    fn add_assign(&mut self, rhs: Self) {
        self.micros += rhs.micros;
    }
}

impl Sub for SimDuration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            micros: self.micros - rhs.micros,
        }
    }
}

impl SubAssign for SimDuration {
    fn sub_assign(&mut self, rhs: Self) {
        self.micros -= rhs.micros;
    }
}

impl Rem for SimDuration {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self {
            micros: self.micros % rhs.micros,
        }
    }
}

impl RemAssign for SimDuration {
    fn rem_assign(&mut self, rhs: Self) {
        self.micros %= rhs.micros;
    }
}
