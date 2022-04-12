use std::f32::consts::TAU;

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
        let t = if self.a < 0.0 {
            None
        } else {
            Some(TAU * (self.a.powi(3) / self.mu).sqrt())
        };
        let rp = h2m / (1.0 + self.e);
        let ra = h2m / (1.0 - self.e);
        Aux { h2m, t, rp, ra }
    }
}

pub struct Aux {
    pub h2m: f32,
    pub t: Option<f32>,
    pub rp: f32,
    pub ra: f32,
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
    pub fn radius(&self, theta: f32) -> f32 {
        self.aux.h2m / (1.0 + self.state.e * theta.cos())
    }

    pub fn theta(&self, time: f32) -> Option<f32> {
        let t = self.aux.t?;
        let me = TAU / t * (time % t);
        let mut e = me;
        for n in 1..=20 {
            e += 2.0 / (n as f32) * j(n, n as f32 * self.state.e) * (n as f32 * me).sin();
        }
        let theta =
            2.0 * ((0.5 * e).tan() * ((1.0 + self.state.e) / (1.0 - self.state.e)).sqrt()).atan();
        Some(theta)
    }
}

fn j(n: i32, x: f32) -> f32 {
    let mut acc = 0.0;
    for k in 0..=20 {
        acc += (-1.0f32).powi(k) * (0.5 * x).powi(n + 2 * k) / (factorial(k) * factorial(n + k))
    }
    acc
}

fn factorial(x: i32) -> f32 {
    (1..x).map(|y| y as f32).product()
}
