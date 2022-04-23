struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

struct InstanceInput {
    [[location(2)]] model_0: vec4<f32>;
    [[location(3)]] model_1: vec4<f32>;
    [[location(4)]] model_2: vec4<f32>;
    [[location(5)]] model_3: vec4<f32>;
    [[location(6)]] albedo: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] albedo: vec3<f32>;
};

struct Viewport {
    view_proj: mat4x4<f32>;
    eye: vec3<f32>;
    forward_xfov: vec4<f32>;
    up_yfov: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> viewport: Viewport;

[[stage(vertex)]]
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.model_0,
        instance.model_1,
        instance.model_2,
        instance.model_3,
    );
    var out: VertexOutput;

    let position = model * vec4<f32>(vertex.position, 1.0);
    out.position = position.xyz;
    out.normal = normalize((model * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.clip_position = viewport.view_proj * position;
    out.albedo = instance.albedo;

    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let dx = dpdx(in.position);
    // dy in window coordinates, positive dpdy means down
    let dy = -dpdy(in.position);
    let normal = normalize(cross(dx, dy));

    let ray = normalize(viewport.eye - in.position);
    let coef = dot(ray, normal);
    return vec4<f32>(in.albedo * coef, 1.0);
}