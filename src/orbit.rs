use std::{cmp::Ordering, f64::consts::TAU};

use glam::{DQuat, DVec2, DVec3};

use crate::time::{SimDuration, SimInstant};

#[derive(Debug, Clone, Copy)]
pub struct Orbit2D {
    // Eccentricity
    e: f64,
    /// Parameter, or semi-latus rectum
    p: f64,
    /// Time at periapsis
    t0: SimInstant,
    /// Gravitational parameter `G * (m1 + m2)`
    grav: f64,
}

impl Orbit2D {
    pub fn new(e: f64, p: f64, t0: SimInstant, grav: f64) -> Self {
        Self { e, p, t0, grav }
    }

    pub fn from_apsides(ra: f64, rp: f64, t0: SimInstant, grav: f64) -> Self {
        let e = (ra - rp) / (ra + rp);
        let p = rp * (1.0 + e);
        Self::new(e, p, t0, grav)
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

    /// Reciprocal of semi-major axis
    fn alpha(&self) -> f64 {
        (1.0 - self.e.powi(2)) / self.p
    }

    /// Specific angular momentum
    fn h(&self) -> f64 {
        (self.p * self.grav).sqrt()
    }

    /// Orbital period
    ///
    /// Note this will only be `Some` if the orbit is elliptical / periodic,
    /// i.e. the eccentricity is less than 1.
    fn period(&self) -> Option<SimDuration> {
        if self.is_elliptic() {
            Some(SimDuration::from_secs_f64(
                TAU * (self.a().powi(3) / self.grav).sqrt(),
            ))
        } else {
            None
        }
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

    pub fn current_state(&self, time: SimInstant) -> State2D {
        let dt = time - self.t0;
        let dt = match self.period() {
            Some(period) => dt % period,
            None => dt,
        };
        let dt_secs = dt.as_secs_f64();

        let chi = self.chi(dt_secs);

        let &Self { grav, .. } = self;
        let alpha = self.alpha();
        let rp = self.rp();
        let r0 = rp * DVec2::X;
        let v0 = self.h() / rp * DVec2::Y;

        let z = alpha * chi.powi(2);

        let f = 1.0 - chi.powi(2) / rp * sc(z);
        let g = dt_secs - chi.powi(3) * ss(z) / grav.sqrt();
        let position = f * r0 + g * v0;
        let r = position.length();

        let df = grav.sqrt() / (r * rp) * (alpha * chi.powi(3) * ss(z) - chi);
        let dg = 1.0 - chi.powi(2) / r * sc(z);
        let velocity = df * r0 + dg * v0;

        State2D {
            position,
            velocity,
            time,
        }
    }
}

pub struct State2D {
    pub position: DVec2,
    pub velocity: DVec2,
    pub time: SimInstant,
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

    pub fn from_current_state(state: &State3D, grav: f64) -> Self {
        let &State3D {
            position: r,
            velocity: v,
            ..
        } = state;
        let r_mag = r.length();
        let v_mag = v.length();

        let h = r.cross(v);
        let n = DVec3::Z.cross(h);
        let e = ((v_mag.powi(2) - grav / r_mag) * r - r_mag * v_mag * v) / grav;
        let h_mag = h.length();
        let n_mag = n.length();
        let e_mag = e.length();

        let inc = (h.z / h_mag).acos();

        let half_lan = (n.x / n_mag).acos();
        let lan = if n.y >= 0.0 { half_lan } else { TAU - half_lan };

        let half_arg_pe = (n.dot(e) / (n_mag * e_mag)).acos();
        let arg_pe = if e.z >= 0.0 {
            half_arg_pe
        } else {
            TAU - half_arg_pe
        };

        let p = h_mag.powi(2) / grav;
        let a = p / (1.0 - e_mag.powi(2));

        let half_theta = (e.dot(r) / (e_mag * r_mag)).acos();
        let theta = if r.dot(v) >= 0.0 {
            half_theta
        } else {
            TAU - half_theta
        };

        let t = match e_mag.partial_cmp(&1.0) {
            Some(Ordering::Less) => {
                // Elliptical
                let ea =
                    2.0 * (((1.0 - e_mag) / (1.0 + e_mag)).sqrt() * (theta / 2.0).tan()).atan();
                let ma = ea - e_mag * ea.sin();
                ma * (a.powi(3) / grav).sqrt()
            }
            Some(Ordering::Greater) => {
                // Hyperbolic
                let fa =
                    2.0 * (((1.0 - e_mag) / (1.0 + e_mag)).sqrt() * (theta / 2.0).tanh()).atanh();
                let ma = e_mag * fa.sinh() - fa;
                ma * (a.powi(3) / grav).abs().sqrt()
            }
            Some(Ordering::Equal) => {
                // Parabolic
                let da = (theta / 2.0).tan();
                let ma = da + da.powi(3) / 3.0;
                let q = p / (1.0 - e_mag);
                ma * (q.powi(3) / grav).sqrt()
            }
            None => panic!("invalid parameter e={}", e),
        };
        let t0 = state.time - SimDuration::from_secs_f64(t);

        Self::new(Orbit2D::new(e_mag, p, t0, grav), arg_pe, inc, lan)
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

    pub fn current_state(&self, time: SimInstant) -> State3D {
        let xf = self.orientation();
        let state_2d = self.shape.current_state(time);
        State3D {
            position: xf * state_2d.position.extend(0.0),
            velocity: xf * state_2d.velocity.extend(0.0),
            time: state_2d.time,
        }
    }
}

pub struct State3D {
    pub position: DVec3,
    pub velocity: DVec3,
    pub time: SimInstant,
}
