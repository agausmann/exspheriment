use std::time::Instant;

use glam::{DVec3, Mat4, Quat, Vec3};
use valet::{Tag, Valet};

use crate::{
    orbit::{Orbit2D, Orbit3D, State3D},
    time::{SimDuration, SimInstant},
};

/// Universal gravitational constant (m^3/kg/s^2)
const G: f64 = 6.67430e-11;

pub struct World {
    bodies: Valet<Body>,
    roots: Vec<Tag<Body>>,
    time: SimInstant,
    last_update: Instant,
    pub body_tags: Vec<Tag<Body>>,
}

impl World {
    pub fn new() -> Self {
        let mut this = Self {
            bodies: Valet::new(),
            roots: Vec::new(),
            time: SimInstant::epoch(),
            last_update: Instant::now(),
            body_tags: vec![],
        };

        let sun = this.add_body(
            &OrbitSpec::Fixed(DVec3::ZERO),
            2.0e30,
            6.957e9,
            // 6.957e8,
        );
        let earth = this.add_body(
            &OrbitSpec::Apsides {
                parent: sun,
                apo: 1.521e11,
                peri: 1.47095e11,
                t0: SimInstant::epoch(),
                arg_pe: 114.20783_f64.to_radians(),
                inc: 1.57869_f64.to_radians(),
                lan: -11.26064_f64.to_radians(),
            },
            5.97237e24,
            6.957e9,
            // 6.365e6,
        );
        let _moon = this.add_body(
            &OrbitSpec::Apsides {
                parent: earth,
                apo: 4.054e8,
                peri: 3.626e8,
                t0: SimInstant::epoch(),
                arg_pe: 0.0,
                inc: 5.145_f64.to_radians(),
                lan: 0.0,
            },
            7.342e22,
            1.736e6,
        );

        this
    }

    fn add_body(&mut self, orbit_spec: &OrbitSpec, mass: f64, radius: f64) -> Tag<Body> {
        let (m1, parent_state) = orbit_spec
            .parent()
            .map(|tag| {
                let parent = &self.bodies[&tag];
                (parent.mass, parent.abs_state)
            })
            .unwrap_or((0.0, State3D::zero(self.time)));
        let trajectory = orbit_spec.to_trajectory(G * (m1 + mass));
        let state = match orbit_spec {
            &OrbitSpec::InitialState { state, .. } => state,
            _ => trajectory.current_state(self.time),
        };
        let abs_state = state.offset_by(&parent_state);

        let tag = self.bodies.insert(Body {
            trajectory,
            abs_state,
            satellites: vec![],
            mass,
            radius,
        });
        if let Some(parent) = orbit_spec.parent() {
            self.bodies[&parent].satellites.push(tag);
        } else {
            self.roots.push(tag);
        }
        self.body_tags.push(tag);
        tag
    }

    fn update_positions(&mut self) {
        let mut pending = self.roots.clone();
        while let Some(tag) = pending.pop() {
            let parent_state = self.bodies[&tag]
                .trajectory
                .parent()
                .map(|parent| self.bodies[&parent].abs_state)
                .unwrap_or(State3D::zero(self.time));
            let body = &mut self.bodies[&tag];
            body.abs_state = body
                .trajectory
                .current_state(self.time)
                .offset_by(&parent_state);
            pending.extend(body.satellites.iter().copied());
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = SimDuration::from((now - self.last_update) * 100000);
        self.time += dt;
        self.last_update = now;

        self.update_positions();
    }

    pub fn body(&self, tag: &Tag<Body>) -> &Body {
        &self.bodies[tag]
    }
}

pub struct Body {
    trajectory: Trajectory,
    abs_state: State3D,
    satellites: Vec<Tag<Body>>,
    mass: f64,
    radius: f64,
}

impl Body {
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::splat(self.radius as f32),
            Quat::IDENTITY,
            self.abs_state.position.as_vec3(),
        )
    }
}

pub enum OrbitSpec {
    Fixed(DVec3),
    InitialState {
        parent: Tag<Body>,
        state: State3D,
    },
    Apsides {
        parent: Tag<Body>,
        apo: f64,
        peri: f64,
        t0: SimInstant,
        arg_pe: f64,
        inc: f64,
        lan: f64,
    },
}

impl OrbitSpec {
    fn parent(&self) -> Option<&Tag<Body>> {
        match self {
            Self::InitialState { parent, .. } | Self::Apsides { parent, .. } => Some(parent),
            _ => None,
        }
    }

    fn to_trajectory(&self, grav: f64) -> Trajectory {
        match self {
            Self::Fixed(position) => Trajectory::Fixed(*position),
            &Self::InitialState { parent, state } => Trajectory::Orbiting {
                parent,
                orbit: Orbit3D::from_current_state(&state, grav),
            },
            &Self::Apsides {
                parent,
                apo,
                peri,
                t0,
                arg_pe,
                inc,
                lan,
            } => Trajectory::Orbiting {
                parent,
                orbit: Orbit3D::new(Orbit2D::from_apsides(apo, peri, t0, grav), arg_pe, inc, lan),
            },
        }
    }
}

enum Trajectory {
    Fixed(DVec3),
    Orbiting { parent: Tag<Body>, orbit: Orbit3D },
}

impl Trajectory {
    fn parent(&self) -> Option<&Tag<Body>> {
        match self {
            Self::Orbiting { parent, .. } => Some(parent),
            _ => None,
        }
    }

    fn current_state(&self, time: SimInstant) -> State3D {
        match self {
            &Self::Fixed(position) => State3D {
                position,
                velocity: DVec3::ZERO,
                time,
            },
            Self::Orbiting { orbit, .. } => orbit.current_state(time),
        }
    }
}
