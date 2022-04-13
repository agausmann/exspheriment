use std::{cmp::Ordering, f32::consts::TAU};

use glam::Vec2;

#[derive(Clone, Copy)]
pub struct State {
    pub mu: f32,
    pub e: f32,
    pub a: f32,
}

impl State {
    pub fn aux(&self) -> Aux {
        // h^2 / mu
        let h2m = self.a * (1.0 - self.e.powi(2));
        let h = (h2m * self.mu).sqrt();

        let t = if self.a < 0.0 {
            None
        } else {
            Some(TAU * (self.a.powi(3) / self.mu).sqrt())
        };
        let alpha = self.a.recip();
        let b = self.a * (1.0 - self.e.powi(2)).abs().sqrt();
        let rp = h2m / (1.0 + self.e);
        let ra = h2m / (1.0 - self.e);

        let r0 = rp * Vec2::X;
        let v0 = h / rp * Vec2::Y;

        Aux {
            h2m,
            h,
            t,
            b,
            rp,
            ra,
            alpha,
            r0,
            v0,
        }
    }
}

pub struct Aux {
    pub r0: Vec2,
    pub v0: Vec2,
    pub h2m: f32,
    pub h: f32,
    pub t: Option<f32>,
    pub b: f32,
    pub rp: f32,
    pub ra: f32,
    pub alpha: f32,
}

pub struct Orbit {
    pub state: State,
    pub aux: Aux,
}

impl Orbit {
    pub fn new(state: State) -> Self {
        Self {
            state,
            aux: state.aux(),
        }
    }

    fn chi(&self, time: f32) -> f32 {
        let &State { mu, .. } = &self.state;
        let &Aux { alpha, rp, .. } = &self.aux;

        let mut chi = mu.sqrt() * alpha.abs() * time;
        // In practice, seems to converge in at most 3 iterations.
        for _ in 0..3 {
            let delta = ((1.0 - alpha * rp) * chi.powi(3) * ss(alpha * chi.powi(2)) + rp * chi
                - mu.sqrt() * time)
                / ((1.0 - alpha * rp) * chi.powi(2) * sc(alpha * chi.powi(2)) + rp);
            chi -= delta;
        }
        chi
    }

    pub fn current_position(&self, time: f32) -> Position {
        let chi = self.chi(time);
        let &State { mu, .. } = &self.state;
        let &Aux {
            alpha, rp, r0, v0, ..
        } = &self.aux;
        let z = alpha * chi.powi(2);

        let f = 1.0 - chi.powi(2) / rp * sc(z);
        let g = time - chi.powi(3) * ss(z) / mu.sqrt();
        let position = f * r0 + g * v0;
        let r = position.length();

        let df = mu.sqrt() / (r * rp) * (alpha * chi.powi(3) * ss(z) - chi);
        let dg = 1.0 - chi.powi(2) / r * sc(z);
        let velocity = df * r0 + dg * v0;

        Position { position, velocity }
    }
}

pub struct Position {
    pub position: Vec2,
    pub velocity: Vec2,
}

fn ss(z: f32) -> f32 {
    let zq = z.abs().sqrt();
    match z.partial_cmp(&0.0) {
        None => f32::NAN,
        Some(Ordering::Equal) => 1.0 / 6.0,
        Some(Ordering::Greater) => (zq - zq.sin()) / zq.powi(3),
        Some(Ordering::Less) => (zq.sinh() - zq) / zq.powi(3),
    }
}

fn sc(z: f32) -> f32 {
    let zq = z.abs().sqrt();
    match z.partial_cmp(&0.0) {
        None => f32::NAN,
        Some(Ordering::Equal) => 0.5,
        Some(Ordering::Greater) => (1.0 - zq.cos()) / z,
        Some(Ordering::Less) => (zq.cosh() - 1.0) / -z,
    }
}
