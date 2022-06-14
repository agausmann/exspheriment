use bevy::math::Vec3;

pub fn slerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let theta = a.angle_between(b);
    (((1.0 - t) * theta).sin() * a + (t * theta).sin() * b) / theta.sin()
}
