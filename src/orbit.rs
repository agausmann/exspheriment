use std::cmp::Ordering;

use glam::{DQuat, DVec2, DVec3};

#[derive(Debug, Clone, Copy)]
pub struct Orbit2D {
    // Eccentricity
    e: f64,
    /// Parameter, or semi-latus rectum
    p: f64,
    /// Gravitational parameter `G * (m1 + m2)`
    grav: f64,
}

impl Orbit2D {
    pub fn new(e: f64, p: f64, grav: f64) -> Self {
        Self { e, p, grav }
    }

    pub fn from_apsides(ra: f64, rp: f64, grav: f64) -> Self {
        let e = (ra - rp) / (ra + rp);
        let p = rp * (1.0 + e);
        Self::new(e, p, grav)
    }

    pub fn is_elliptic(&self) -> bool {
        self.e < 1.0
    }

    pub fn is_parabolic(&self) -> bool {
        self.e == 1.0
    }

    pub fn is_hyperbolic(&self) -> bool {
        self.e > 1.0
    }

    /// Apoapsis
    pub fn ra(&self) -> f64 {
        self.p / (1.0 - self.e)
    }

    /// Periapsis
    pub fn rp(&self) -> f64 {
        self.p / (1.0 + self.e)
    }

    /// Semi-major axis
    pub fn a(&self) -> f64 {
        self.p / (1.0 - self.e.powi(2))
    }

    /// Semi-minor axis
    pub fn b(&self) -> f64 {
        self.p / (1.0 - self.e.powi(2)).sqrt()
    }

    pub fn radius_at(&self, angle: f64) -> f64 {
        self.p / (1.0 + self.e * angle.cos())
    }

    /// Reciprocal of semi-major axis:
    /// > 0: Ellipse
    /// = 0: Parabola
    /// < 0: Hyperbola
    fn alpha(&self) -> f64 {
        (1.0 - self.e.powi(2)) / self.p
    }

    /// Specific angular momentum
    fn h(&self) -> f64 {
        (self.p * self.grav).sqrt()
    }

    fn chi(&self, time: f64) -> f64 {
        let &Self { grav, .. } = self;
        let alpha = self.alpha();
        let rp = self.rp();

        let mut chi = grav.sqrt() * alpha.abs() * time;
        for _ in 0..100 {
            let delta = ((1.0 - alpha * rp) * chi.powi(3) * ss(alpha * chi.powi(2)) + rp * chi
                - grav.sqrt() * time)
                / ((1.0 - alpha * rp) * chi.powi(2) * sc(alpha * chi.powi(2)) + rp);
            chi -= delta;
            if delta.abs() < 1e-10 {
                break;
            }
        }
        chi
    }

    pub fn current_state(&self, time: f64) -> State2D {
        let chi = self.chi(time);

        let &Self { grav, .. } = self;
        let alpha = self.alpha();
        let rp = self.rp();
        let r0 = rp * DVec2::X;
        let v0 = self.h() / rp * DVec2::Y;

        let z = alpha * chi.powi(2);

        let f = 1.0 - chi.powi(2) / rp * sc(z);
        let g = time - chi.powi(3) * ss(z) / grav.sqrt();
        let position = f * r0 + g * v0;
        let r = position.length();

        let df = grav.sqrt() / (r * rp) * (alpha * chi.powi(3) * ss(z) - chi);
        let dg = 1.0 - chi.powi(2) / r * sc(z);
        let velocity = df * r0 + dg * v0;

        State2D { position, velocity }
    }
}

pub struct State2D {
    pub position: DVec2,
    pub velocity: DVec2,
}

fn ss(z: f64) -> f64 {
    let zq = z.abs().sqrt();
    match z.partial_cmp(&0.0) {
        None => f64::NAN,
        Some(Ordering::Equal) => 1.0 / 6.0,
        Some(Ordering::Greater) => (zq - zq.sin()) / zq.powi(3),
        Some(Ordering::Less) => (zq.sinh() - zq) / zq.powi(3),
    }
}

fn sc(z: f64) -> f64 {
    let zq = z.abs().sqrt();
    match z.partial_cmp(&0.0) {
        None => f64::NAN,
        Some(Ordering::Equal) => 0.5,
        Some(Ordering::Greater) => (1.0 - zq.cos()) / z,
        Some(Ordering::Less) => (zq.cosh() - 1.0) / -z,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Orbit3D {
    shape: Orbit2D,

    /// Argument of periapsis relative to ascending node (radians)
    arg_pe: f64,

    /// Inclination (radians)
    inc: f64,

    /// Longitude of ascending node relative to +X (radians)
    lan: f64,
}

impl Orbit3D {
    pub fn new(shape: Orbit2D, arg_pe: f64, inc: f64, lan: f64) -> Self {
        Self {
            shape,
            arg_pe,
            inc,
            lan,
        }
    }

    pub fn from_current_state(position: DVec3, velocity: DVec3) -> Self {
        todo!()
    }

    pub fn shape(&self) -> &Orbit2D {
        &self.shape
    }

    pub fn a_vector(&self) -> DVec3 {
        self.shape.a() * (self.orientation() * DVec3::X)
    }

    pub fn b_vector(&self) -> DVec3 {
        self.shape.b() * (self.orientation() * DVec3::Y)
    }

    fn orientation(&self) -> DQuat {
        return DQuat::from_rotation_z(self.lan)
            * DQuat::from_rotation_x(self.inc)
            * DQuat::from_rotation_z(self.arg_pe);
    }

    pub fn current_state(&self, time: f64) -> State3D {
        let xf = self.orientation();
        let state_2d = self.shape.current_state(time);
        State3D {
            position: xf * state_2d.position.extend(0.0),
            velocity: xf * state_2d.velocity.extend(0.0),
        }
    }
}

pub struct State3D {
    pub position: DVec3,
    pub velocity: DVec3,
}
